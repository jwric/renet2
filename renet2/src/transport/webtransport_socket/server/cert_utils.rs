use anyhow::Error;
use rcgen::{CertificateParams, DistinguishedName, DnType, Ia5String, IsCa, KeyIdMethod, KeyPair, SanType, PKCS_ECDSA_P256_SHA256};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use time::{ext::NumericalDuration, OffsetDateTime};

use std::{path::PathBuf, str::FromStr};

use crate::transport::ServerCertHash;

/// Generates a self-signed certificate for use in [`WebTransportConfig`].
///
/// The [`PrivateKey`] should not be publicized.
pub fn generate_self_signed_certificate(params: CertificateParams) -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>), rcgen::Error> {
    let key_pair = KeyPair::generate()?;
    let rc_certificate = params.self_signed(&key_pair)?;

    let certificate = rc_certificate.der().clone();
    let privkey = PrivateKeyDer::Pkcs8(key_pair.serialize_der().into());

    Ok((certificate, privkey))
}

/// Generates a self-signed certificate for use in [`WebTransportConfig`].
///
/// The certificate generated is opinionated based on how it is expected to be used.
/// - The validity period is set to 2 weeks from now to support [`ServerCertHash`].
/// - ECDSA is used, not RSA.
///
/// The [`PrivateKey`] should not be publicized.
pub fn generate_self_signed_certificate_opinionated(
    subject_alt_names: impl IntoIterator<Item = String>,
) -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>), rcgen::Error> {
    let not_before = OffsetDateTime::now_utc().saturating_sub(1.hours()); //adjust for client system time variance
    let not_after = not_before.saturating_add(2.weeks().saturating_sub(1.minutes())); //less than 2 weeks
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, "renet2 self signed cert");

    let subject_alt_names = subject_alt_names
        .into_iter()
        .map(|s| match s.parse() {
            Ok(ip) => SanType::IpAddress(ip),
            Err(_) => SanType::DnsName(Ia5String::from_str(s.as_str()).unwrap()),
        })
        .collect::<Vec<_>>();

    let mut params = CertificateParams::default(); //the params is `non_exhaustive` so we need to default construct
    // params.alg = &PKCS_ECDSA_P256_SHA256;
    params.not_before = not_before;
    params.not_after = not_after;
    params.serial_number = None;
    params.subject_alt_names = subject_alt_names;
    params.distinguished_name = distinguished_name;
    params.is_ca = IsCa::NoCa;
    params.key_usages = Vec::new();
    params.extended_key_usages = Vec::new();
    params.name_constraints = None;
    params.crl_distribution_points = Vec::new();
    params.custom_extensions = Vec::new();
    // params.key_pair = None;
    params.use_authority_key_identifier_extension = false;
    params.key_identifier_method = KeyIdMethod::Sha256;

    generate_self_signed_certificate(params)
}

/// Loads a certificate and private key from the file system.
///
/// The certificate and key must be DER encoded.
pub fn get_certificate_and_key_from_files(cert: PathBuf, key: PathBuf) -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>), Error> {
    let cert = CertificateDer::from(std::fs::read(cert)?);
    let key = PrivateKeyDer::Pkcs8(std::fs::read(key)?.into());
    Ok((cert, key))
}

/*
/// SPKI fingerprint is needed when launching a Chrome browser with a custom cert chain.
fn _spki_fingerprint(cert: &Certificate) -> Option<spki::FingerprintBytes> {
    let cert = x509_cert::Certificate::from_der(&cert.0).ok()?;
    let fingerprint = cert
        .tbs_certificate
        .subject_public_key_info
        .fingerprint_bytes()
        .ok()?;
    Some(fingerprint)
}

fn _spki_fingerprint_base64(cert: &Certificate) -> Option<String> {
    _spki_fingerprint(cert)
        .map(|fingerprint| base64::engine::general_purpose::STANDARD.encode(fingerprint))
}
*/

/// Gets a [`ServerCertHash`] from a [`Certificate`].
pub fn get_server_cert_hash(cert: &CertificateDer) -> ServerCertHash {
    let hash = hmac_sha256::Hash::hash(cert.as_ref());
    ServerCertHash { hash }
}
