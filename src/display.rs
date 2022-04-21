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

use crate::chain::{Chain, CertificateFile, CertificateRequestFile};
use crate::options::Options;

/// Default display mode handler.
pub fn default(chains: Vec<Chain>)
{
    for chain in chains
    {
        println!("{}", get_display_name(&chain));

        match chain.key {
            Some(key) => println!("  * Key: {}", key.path),
            _ => continue
        }

        print!("  * CSR: ");

        match chain.request {
            Some(request) => println!("{}", request.path),
            _ => println!("n/a")
        }

        print!("  * Certificates: ");

        if chain.certificates.len() == 0
        {
            println!("n/a");
        }
        else
        {
            println!();

            for certificate in chain.certificates
            {
                let mut indentation = 4;

                print_indentation(indentation);

                print!("- {}", certificate.path);

                if certificate.self_signed
                {
                    println!(" (self-signed)");

                    break;
                }

                println!();

                // Print chain of signing certificates recursively.
                for signing_certificate in certificate.signing_certificate_chain()
                {
                    indentation += 2;

                    print_indentation(indentation);

                    println!("> {}", signing_certificate.path);
                }
            }
        }
    }
}

/// OneLine display mode handler.
pub fn oneline(chains: Vec<Chain>)
{
    if !Options::new().suppress_oneline_header
    {
        println!("name key request certificate_chain");
    }

    for chain in chains
    {
        print!("{}", get_display_name(&chain));

        match chain.key {
            Some(key) => print!(" {}", key.path),
            _ => continue
        }

        match chain.request {
            Some(request) => print!(" {}", request.path),
            _ => print!(" -")
        }

        if chain.certificates.len() == 0
        {
            println!(" -");

            continue;
        }

        for certificate in chain.certificates
        {
            print!(" {}", certificate.path);

            if certificate.self_signed
            {
                print!("|(self-signed)");

                break;
            }

            // Print chain of signing certificates recursively.
            for signing_certificate in certificate.signing_certificate_chain()
            {
                print!("|{}", signing_certificate.path);
            }
        }

        println!();
    }
}

/// Get display name for chain by checking multiple sources.
fn get_display_name(chain: &Chain) -> String
{

    if let Some(certificate) = &chain.certificates.get(0)
    {
        if let Some(name) = get_display_name_from_certificate(&certificate)
        {
            return name;
        }
    }

    if let Some(request) = &chain.request
    {
        if let Some(name) = get_display_name_from_request(&request)
        {
            return name;
        }
    }

    "(unknown)".to_string()
}

/// Get display name from a certificate signing request.
fn get_display_name_from_request(request: &CertificateRequestFile) -> Option<String>
{
    if let Some(common_name) = request.common_name()
    {
        if let Some(line) = common_name.data().as_utf8().unwrap().lines().last()
        {
            return Some(line.to_string());
        }
    }

    None
}

/// Get display name from a certificate signing request.
fn get_display_name_from_certificate(certificate: &CertificateFile) -> Option<String>
{
    // Prefer subject alternative name values.
    let x509 = &certificate.certificate;

    if let Some(general_name_stack) = x509.subject_alt_names()
    {
        if ! general_name_stack.is_empty()
        {
            // Prefer non-www values.
            for general_name in &general_name_stack
            {
                if let Some(dnsname) = general_name.dnsname()
                {
                    if ! dnsname.starts_with("www.")
                    {
                        return Some(dnsname.to_string());
                    }
                }
            }

            // Fall back to first entry.
            if let Some(general_name) = general_name_stack.get(0)
            {
                if let Some(dnsname) = general_name.dnsname()
                {
                    return Some(dnsname.to_string());
                }
            }
        }
    }

    // Fall back to common name value.
    if let Some(common_name) = certificate.common_name()
    {
        if let Some(line) = common_name.data().as_utf8().unwrap().lines().last()
        {
            return Some(line.to_string());
        }
    }

    None
}

/// Print a given number spaces for intentation.
fn print_indentation(spaces: i32)
{
    for _ in 1..=spaces { print!(" "); }
}

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn gets_display_name_prefers_certificate()
    {
        use crate::chain;

        let mut chain = chain::Chain::new();

        let path = "samples/self_signed_san.key";
        let contents = chain::get_file_contents(&path).unwrap();
        let key = chain::str_to_private_key(&contents).unwrap();
        chain.key = Some(chain::PrivateKeyFile::new(path, key));

        let path = "samples/self_signed_san.csr";
        let contents = chain::get_file_contents(&path).unwrap();
        let x509req = chain::str_to_x509req(&contents).unwrap();
        chain.request = Some(chain::CertificateRequestFile::new(path, x509req));

        let path = "samples/self_signed_san.crt";
        let contents = chain::get_file_contents(&path).unwrap();
        let x509 = chain::str_to_x509(&contents).unwrap();
        chain.certificates = vec![chain::CertificateFile::new(path, x509)];

        assert_eq!(get_display_name(&chain), "san.example.com".to_string());
    }

    #[test]
    fn gets_display_name_falls_back_to_request()
    {
        use crate::chain;

        let mut chain = chain::Chain::new();

        let path = "samples/self_signed_san_no_cert.key";
        let contents = chain::get_file_contents(&path).unwrap();
        let key = chain::str_to_private_key(&contents).unwrap();
        chain.key = Some(chain::PrivateKeyFile::new(path, key));

        let path = "samples/self_signed_san_no_cert.csr";
        let contents = chain::get_file_contents(&path).unwrap();
        let x509req = chain::str_to_x509req(&contents).unwrap();
        chain.request = Some(chain::CertificateRequestFile::new(path, x509req));

        assert_eq!(get_display_name(&chain), "example.com".to_string());
    }

    #[test]
    fn reads_common_name_from_certificate_request()
    {
        use crate::chain;

        let path = "samples/self_signed.csr";
        let contents = chain::get_file_contents(&path).unwrap();
        let x509req = chain::str_to_x509req(&contents).unwrap();
        let request = chain::CertificateRequestFile::new(path, x509req);

        match get_display_name_from_request(&request)
        {
            Some(display_name) => assert_eq!(display_name, "example.com"),
            _ => assert!(false)
        }
    }

    #[test]
    fn reads_subject_alternative_name_from_certificate_request()
    {
        use crate::chain;

        let path = "samples/self_signed_san.crt";
        let contents = chain::get_file_contents(&path).unwrap();
        let x509 = chain::str_to_x509(&contents).unwrap();
        let certificate = chain::CertificateFile::new(path, x509);

        match get_display_name_from_certificate(&certificate)
        {
            Some(display_name) => assert_eq!(display_name, "san.example.com"),
            _ => assert!(false)
        }
    }

    #[test]
    fn reads_subject_alternative_name_from_certificate_request_with_www_fallback()
    {
        use crate::chain;

        let path = "samples/self_signed_san_www_only.crt";
        let contents = chain::get_file_contents(&path).unwrap();
        let x509 = chain::str_to_x509(&contents).unwrap();
        let certificate = chain::CertificateFile::new(path, x509);

        match get_display_name_from_certificate(&certificate)
        {
            Some(display_name) => assert_eq!(display_name, "www.san.example.com"),
            _ => assert!(false)
        }
    }
}
