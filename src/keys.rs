// Don't warn on unused code for this module.
#![allow(dead_code)]

use openssl::rsa::Rsa;
use openssl::x509::{X509, X509Req};
use openssl::x509::{X509Builder, X509ReqBuilder, X509NameBuilder};
use openssl::pkey::{PKey, Private, Public};
use openssl::nid::Nid;
use crate::chain;

/**
 * Generate SSL keypairs for testing purposes.
 */
pub fn generate() -> (Rsa<Private>, X509Req, X509)
{
    generate_with_sans(vec![])
}

/**
 * Generate SSL keypairs for testing purposes, with a given list of values for
 * the Subject Alternative Name X509 extension.
 */
pub fn generate_with_sans(sans: Vec<&str>) -> (Rsa<Private>, X509Req, X509)
{
    let rsa: Rsa<Private> = Rsa::generate(2048).unwrap();

    let private_key = private_key(&rsa);

    let public_key = PKey::public_key_from_pem(&rsa.public_key_to_pem().unwrap()).unwrap();

    (
        private_key,
        certificate_request(&public_key, Some(sans)),
        certificate(&public_key)
    )
}

fn private_key(rsa: &Rsa<Private>) -> Rsa<Private>
{
    let pem: Vec<u8> = rsa.private_key_to_pem().unwrap();

    let contents = String::from_utf8(pem).unwrap();

    chain::str_to_private_key(&contents).unwrap()
}

fn certificate_request(pkey: &PKey<Public>, sans: Option<Vec<&str>>) -> X509Req
{
    let mut req_builder = X509ReqBuilder::new().unwrap();

    req_builder.set_pubkey(pkey).unwrap();

    let mut name_builder = X509NameBuilder::new().unwrap();

    if let Some(vec) = sans
    {
        for name in vec
        {
            name_builder.append_entry_by_nid(Nid::SUBJECT_ALT_NAME, name).unwrap();
        }
    }

    let names = name_builder.build();

    req_builder.set_subject_name(names.as_ref()).unwrap();

    req_builder.build()
}

fn certificate(pkey: &PKey<Public>) -> X509
{
    let mut cert = X509Builder::new().unwrap();

    cert.set_pubkey(&pkey).unwrap();

    cert.build()
}