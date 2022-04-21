//! Copyright (C) 2022 Gaz J.
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fs;
use std::io;
use openssl::rsa::Rsa;
use openssl::x509::{X509, X509NameEntryRef, X509Req};
use openssl::pkey::{Public, Private};
use openssl::error::ErrorStack;
use openssl::nid::Nid;

#[path = "compare.rs"] mod compare;

/// Chain struct contains instances of related items.
pub struct Chain
{
    pub name: Option<String>,
    pub key: Option<PrivateKeyFile>,
    pub request: Option<CertificateRequestFile>,
    pub certificates: Vec<CertificateFile>,
}

impl Chain
{
    pub fn new() -> Self
    {
        Chain { name: None, key: None, request: None, certificates: vec![] }
    }
}

/// Represents private key files as the foundation of each Chain.
pub struct PrivateKeyFile
{
    pub path: String,
    pub rsa: Rsa<Private>,
}

impl PrivateKeyFile
{
    pub fn new(path: &str, rsa: Rsa<Private>) -> Self
    {
        PrivateKeyFile { path: path.to_string(), rsa }
    }
}

/// Represents a certificate signing request (if found).
pub struct CertificateRequestFile
{
    pub path: String,
    pub request: X509Req,
}

impl CertificateRequestFile
{
    pub fn new(path: &str, request: X509Req) -> Self
    {
        CertificateRequestFile { path: path.to_string(), request }
    }

    pub fn to_rsa(&self) -> Result<Rsa<Public>, ErrorStack>
    {
        self.request.public_key().unwrap().rsa()
    }

    pub fn common_name(&self) -> Option<&X509NameEntryRef>
    {
        self.request.subject_name().entries_by_nid(Nid::COMMONNAME).last()
    }
}

/// Represents all X509 certificates found, including intermediate and
/// root signing certificates in the chain.
#[derive(Clone)]
pub struct CertificateFile
{
    pub path: String,
    pub certificate: X509,
    pub signing_certificate: Option<Box<CertificateFile>>,
    pub self_signed: bool
}

impl CertificateFile
{
    pub fn new(path: &str, certificate: X509) -> Self
    {
        CertificateFile {
            path: path.to_string(),
            certificate,
            signing_certificate: None,
            self_signed: false
        }
    }

    pub fn to_rsa(&self) -> Result<Rsa<Public>, ErrorStack>
    {
        self.certificate.public_key().unwrap().rsa()
    }

    pub fn signing_certificate_chain(&self) -> Vec<Box<CertificateFile>>
    {
        if self.self_signed { return vec![]; }

        let mut certificate_chain: Vec<Box<CertificateFile>> = vec![];

        let mut certificate = self;

        while let Some(signing_certificate) = &certificate.signing_certificate
        {
            certificate_chain.push(signing_certificate.clone());

            certificate = signing_certificate;

            continue;
        }

        certificate_chain
    }

    pub fn common_name(&self) -> Option<&X509NameEntryRef>
    {
        self.certificate.subject_name().entries_by_nid(Nid::COMMONNAME).last()
    }
}

/// Begin building each Chain instance.
pub fn build(paths: Vec<String>) -> Result<Vec<Chain>, String>
{
    let mut chains = vec![];

    initialize(&mut chains, &paths);

    attach_certificate_signing_requests(&mut chains, &paths);

    attach_certificates(&mut chains, &paths);

    attach_signing_certificates(&mut chains, &paths);

    Ok(chains)
}

/// Initialize Chains, creating one for each private key.
fn initialize(chains: &mut Vec<Chain>, paths: &Vec<String>)
{
    for path in paths
    {
        let contents = get_file_contents(&path);

        if contents.is_err() { continue; }

        let contents = contents.unwrap();

        if let Ok(rsa) = str_to_private_key(&contents)
        {
            let key = PrivateKeyFile::new(&path, rsa);

            let mut chain = Chain::new();

            chain.key = Some(key);

            chains.push(chain);
        }
    }
}

/// Locate certificate signing requests for all existing chains.
fn attach_certificate_signing_requests(chains: &mut Vec<Chain>, paths: &Vec<String>)
{
    for chain in chains
    {
        for path in paths
        {
            let contents = get_file_contents(&path);

            if contents.is_err() { continue; }

            let contents = contents.unwrap();

            let request = str_to_x509req(&contents);

            if request.is_err() { continue; }

            let request = CertificateRequestFile::new(&path, request.unwrap());

            if let Some(key) = &chain.key
            {
                let rsa = request.to_rsa().unwrap();

                if compare::private_to_public(&key.rsa, &rsa).is_ok()
                {
                    chain.request = Some(request);

                    break;
                }
            }
        }
    }
}

/// Locate certificates for all existing chains.
fn attach_certificates(chains: &mut Vec<Chain>, paths: &Vec<String>)
{
    for chain in chains
    {
        for certificate in find_certificates(paths)
        {
            if let Some(key) = &chain.key
            {
                let rsa = certificate.to_rsa().unwrap();

                if compare::private_to_public(&key.rsa, &rsa).is_ok()
                {
                    chain.certificates.push(certificate);
                }
            }
        }
    }
}

/// Locate signing certificates for all existing chains.
fn attach_signing_certificates(chains: &mut Vec<Chain>, paths: &Vec<String>)
{
    for chain in chains
    {
        // Iterate (mutably) over known certificates, looking for
        // signing certificates for each.
        for certificate in chain.certificates.iter_mut()
        {
            attach_signing_certificate_chain(certificate, &paths);
        }
    }
}

/// Recursively apply signing certificates.
fn attach_signing_certificate_chain(certificate: &mut CertificateFile, paths: &Vec<String>)
{
    let mut certificates = find_certificates(&paths);

    for signing_certificate in certificates.iter_mut()
    {
        if let Ok(()) = compare::certificate_to_signing_certificate(
            &certificate.certificate,
            &signing_certificate.certificate
        )
        {
            // Check if certificate is self-signed.
            if certificate.certificate.signature().as_slice()
                == signing_certificate.certificate.signature().as_slice()
            {
                certificate.self_signed = true;

                return;
            }

            // Copy signing certificate into certificate.signing_certificate.
            certificate.signing_certificate = Some(Box::new(signing_certificate.clone()));

            // Call this function recursively, but with signing_certificate
            // as the first argument.
            attach_signing_certificate_chain(signing_certificate, &paths)
        }
    }
}

/// Create vector containing all certificate files, including
/// those which aren't associated with any known chain.
fn find_certificates(paths: &Vec<String>) -> Vec<CertificateFile>
{
    let mut certificates: Vec<CertificateFile> = vec![];

    for path in paths
    {
        let contents = get_file_contents(&path);

        if contents.is_err() { continue; }

        let contents = contents.unwrap();

        let certificate = str_to_x509(&contents);

        if certificate.is_err() { continue; }

        let certificate = CertificateFile::new(&path, certificate.unwrap());

        certificates.push(certificate);
    }

    certificates
}

/// Converts string slices to private keys.
pub fn str_to_private_key(contents: &str) -> Result<Rsa<Private>, ErrorStack>
{
    Rsa::private_key_from_pem(contents.as_bytes())
}

/// Converts string slices to X509 certificate requests.
pub fn str_to_x509req(contents: &str) -> Result<X509Req, ErrorStack>
{
    X509Req::from_pem(contents.as_bytes())
}

/// Converts string slices to X509 certificates.
pub fn str_to_x509(contents: &str) -> Result<X509, ErrorStack>
{
    X509::from_pem(contents.as_bytes())
}

/// Wrapper for file reading operation, in case this is handled
/// differently later.
pub fn get_file_contents(path: &str) -> Result<String, io::Error>
{
    fs::read_to_string(path)
}

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn reads_private_keys()
    {
        let paths = vec!["samples/ca_signed.key", "samples/self_signed.key"];

        for path in paths
        {
            let contents = get_file_contents(path).unwrap();

            assert!(str_to_private_key(&contents).is_ok());
        }
    }

    #[test]
    fn reads_certificate_requests()
    {
        let paths = vec!["samples/ca_signed.csr", "samples/self_signed.csr"];

        for path in paths
        {
            let contents = get_file_contents(path).unwrap();

            let request = CertificateRequestFile::new(&path, str_to_x509req(&contents).unwrap());

            assert!(request.to_rsa().is_ok());
        }
    }

    #[test]
    fn reads_certificates()
    {
        let paths = vec!["samples/ca_signed.crt", "samples/self_signed.crt"];

        for path in paths
        {
            let contents = get_file_contents(path).unwrap();

            let certificate = CertificateFile::new(&path, str_to_x509(&contents).unwrap());

            assert!(certificate.to_rsa().is_ok());
        }
    }
}
