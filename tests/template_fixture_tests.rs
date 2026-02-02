use notenest::{ClinicianSoap, PatientView};

#[test]
fn patient_view_fixture_deserializes() {
    let payload = include_str!("../fixtures/examples/patient_view.example.json");
    let parsed: PatientView = serde_json::from_str(payload).expect("patient view fixture");
    assert!(!parsed.main_concern.is_empty());
}

#[test]
fn clinician_soap_fixture_deserializes() {
    let payload = include_str!("../fixtures/examples/clinician_soap.example.json");
    let parsed: ClinicianSoap = serde_json::from_str(payload).expect("clinician soap fixture");
    assert!(!parsed.s.is_empty());
}
