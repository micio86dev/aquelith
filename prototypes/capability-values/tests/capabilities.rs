use capability_values::capabilities::{fetch, Denied, Root};
use capability_values::string_tokens::{StringToken, TokenTable};

#[test]
fn string_token_can_be_forged_by_anyone_who_knows_the_string() {
    let mut table = TokenTable::default();
    table.grant("net", "api.example");

    // This code was never granted anything; it only knows the token's text.
    let forged = StringToken::new("net");
    assert!(table.check(&forged, "api.example"), "forged token accepted");
}

#[test]
fn capability_denies_hosts_outside_its_scope() {
    let root = Root::root();
    let net = root.net(&["api.example"]);

    let err = net.connect("evil.example").unwrap_err();
    assert_eq!(err, Denied::NotInScope { host: "evil.example".into() });
}

#[test]
fn capability_cannot_be_widened_by_attenuation() {
    let root = Root::root();
    let net = root.net(&["api.example"]);

    let err = net.narrow(&["api.example", "other.example"]).unwrap_err();
    assert_eq!(err, Denied::NotInScope { host: "other.example".into() });
}

#[test]
fn attenuation_to_a_subset_is_allowed() {
    let root = Root::root();
    let net = root.net(&["api.example", "cdn.example"]);

    let narrowed = net.narrow(&["api.example"]).expect("subset is allowed");
    assert!(narrowed.connect("api.example").is_ok());
    assert!(narrowed.connect("cdn.example").is_err());
}

#[test]
fn function_reaching_network_takes_the_capability_as_a_parameter() {
    let root = Root::root();
    let net = root.net(&["api.example"]);

    assert_eq!(fetch(&net, "api.example"), Ok("connected:api.example".to_string()));
    assert!(fetch(&net, "evil.example").is_err());
}
