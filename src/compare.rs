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

use openssl::rsa::Rsa;
use openssl::x509::X509;
use openssl::pkey::{Public, Private};

/// Compares private and public key files.
pub fn private_to_public<'a>(
    rsa_private: &Rsa<Private>,
    rsa_public: &Rsa<Public>
) -> Result<(), &'a str>
{
    if rsa_private.n() != rsa_public.n()
    {
        return Err("Key file mismatch");
    }
    Ok(())
}

/// Compares X509 certificates to see if one is signed by the other.
pub fn certificate_to_signing_certificate<'a>(
    certificate: &X509,
    signing_certificate: &X509
) -> Result<(), &'a str>
{
    let signing_key = signing_certificate.public_key().unwrap();

    if !certificate.verify(&signing_key).unwrap()
    {
        return Err("Certificate not signed by given signing certificate");
    }
    Ok(())
}

#[cfg(test)]

mod test
{
    use super::*;

    #[test]
    fn identifies_private_and_public_matches()
    {
        use crate::keys;

        let (key, req, cert) = keys::generate();

        let req: Rsa<Public> = req.public_key().unwrap().rsa().unwrap();
        let cert: Rsa<Public> = cert.public_key().unwrap().rsa().unwrap();

        assert!(private_to_public(&key, &req).is_ok());
        assert!(private_to_public(&key, &cert).is_ok());
    }

    #[test]
    fn identifies_private_and_public_mismatches()
    {
        use crate::keys;

        let (key, _, _) = keys::generate();
        let (_, req, cert) = keys::generate();

        let req: Rsa<Public> = req.public_key().unwrap().rsa().unwrap();
        let cert: Rsa<Public> = cert.public_key().unwrap().rsa().unwrap();

        assert!(private_to_public(&key, &req).is_err());
        assert!(private_to_public(&key, &cert).is_err());
    }

    #[test]
    fn identifies_self_signed_certificate_to_signing_certificate_matches()
    {
        use crate::chain;

        let cert_path = chain::get_file_contents("samples/self_signed.crt").unwrap();
        let cert = chain::str_to_x509(&cert_path).unwrap();

        assert!(certificate_to_signing_certificate(&cert, &cert).is_ok());
    }

    #[test]
    fn identifies_ca_signed_certificate_to_signing_certificate_matches()
    {
        use crate::chain;

        let cert_path = chain::get_file_contents("samples/ca_signed.crt").unwrap();
        let cert = chain::str_to_x509(&cert_path).unwrap();

        let ca_cert_path = chain::get_file_contents("samples/intermediate_ca.crt").unwrap();
        let ca_cert = chain::str_to_x509(&ca_cert_path).unwrap();

        assert!(certificate_to_signing_certificate(&cert, &ca_cert).is_ok());
    }

    #[test]
    fn identifies_self_signed_certificate_to_signing_certificate_mismatches()
    {
        use crate::chain;

        let cert_path = chain::get_file_contents("samples/self_signed.crt").unwrap();
        let cert = chain::str_to_x509(&cert_path).unwrap();

        let ca_cert_path = chain::get_file_contents("samples/intermediate_ca.crt").unwrap();
        let ca_cert = chain::str_to_x509(&ca_cert_path).unwrap();

        assert!(certificate_to_signing_certificate(&cert, &ca_cert).is_err());
    }

    #[test]
    fn identifies_ca_signed_certificate_to_signing_certificate_mismatches()
    {
        use crate::chain;

        let cert_path = chain::get_file_contents("samples/ca_signed.crt").unwrap();
        let cert = chain::str_to_x509(&cert_path).unwrap();

        let ca_cert_path = chain::get_file_contents("samples/intermediate_ca.crt").unwrap();
        let ca_cert = chain::str_to_x509(&ca_cert_path).unwrap();

        assert!(certificate_to_signing_certificate(&ca_cert, &cert).is_err());
    }
}
