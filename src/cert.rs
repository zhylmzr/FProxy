use std::{error::Error, fs::File};

use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::{extension::SubjectAlternativeName, X509Builder, X509Name, X509},
};

pub struct Certificate {
    pub cert: X509,
    pub key: PKey<Private>,
}

impl Certificate {
    pub fn new(cert_file: &str, key_file: &str) -> Result<Self, std::io::Error> {
        let mut cert_file = File::open(cert_file)?;
        let mut cert = vec![];
        std::io::copy(&mut cert_file, &mut cert)?;
        let cert = X509::from_pem(&cert)?;

        let mut key_file = File::open(key_file)?;
        let mut key = vec![];
        std::io::copy(&mut key_file, &mut key)?;
        let key = PKey::from_rsa(Rsa::private_key_from_pem(&key)?)?;

        Ok(Self { cert, key })
    }
}

pub fn make_cert_from_ca(domain: &str, ca: &Certificate) -> Result<X509, Box<dyn Error>> {
    let x509_serial = {
        let mut serial_number = BigNum::new()?;
        serial_number.rand(128, MsbOption::MAYBE_ZERO, false)?;
        serial_number.to_asn1_integer()?
    };
    let x509_not_before = Asn1Time::days_from_now(0)?;
    let x509_not_after = Asn1Time::days_from_now(365)?;
    let mut x509_alt_name = SubjectAlternativeName::new();
    x509_alt_name.dns(domain);
    let mut host_name = X509Name::builder()?;
    host_name.append_entry_by_text("CN", domain)?;
    let host_name = host_name.build();

    let mut x509 = X509Builder::new()?;
    x509.set_version(2)?;
    x509.set_serial_number(&x509_serial)?;
    x509.set_pubkey(&ca.key)?;
    x509.set_not_before(&x509_not_before)?;
    x509.set_not_after(&x509_not_after)?;
    x509.set_subject_name(&host_name)?;
    x509.append_extension(x509_alt_name.build(&x509.x509v3_context(Some(&ca.cert), None))?)?;
    x509.set_issuer_name(&ca.cert.issuer_name())?;
    x509.sign(&ca.key, MessageDigest::sha256())?;
    let x509 = x509.build();
    Ok(x509)
}
