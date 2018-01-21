extern crate navitia_model;
extern crate tempdir;

use navitia_model::gtfs;
use tempdir::TempDir;
use std::fs::File;
use std::io::prelude::*;

#[test]
fn load_minimal_agency() {
    let agency_content = "agency_name,agency_url,agency_timezone\n
    My agency,http://my-agency_url.com,Europe/London";
    let tmp_dir = TempDir::new("navitia_model_tests").expect("create temp dir");
    let file_path = tmp_dir.path().join("agency.txt");
    let mut f = File::create(&file_path).unwrap();
    f.write_all(agency_content.as_bytes()).unwrap();

    let (networks, companies) = gtfs::read_agency(tmp_dir.path());
    tmp_dir.close().expect("delete temp dir");
    assert_eq!(1, networks.len());
    let agency = networks.iter().next().unwrap().1;
    assert_eq!("default_agency_id", agency.id);
    assert_eq!(1, companies.len());
}

#[test]
fn load_standard_agency() {
    let agency_content = "agency_id,agency_name,agency_url,agency_timezone\n
id_1,My agency,http://my-agency_url.com,Europe/London";
    let tmp_dir = TempDir::new("navitia_model_tests").expect("create temp dir");
    let file_path = tmp_dir.path().join("agency.txt");
    let mut f = File::create(&file_path).unwrap();
    f.write_all(agency_content.as_bytes()).unwrap();

    let (networks, companies) = gtfs::read_agency(tmp_dir.path());
    tmp_dir.close().expect("delete temp dir");
    assert_eq!(1, networks.len());
    assert_eq!(1, companies.len());
}

#[test]
fn load_complete_agency() {
    let agency_content = "agency_id,agency_name,agency_url,agency_timezone,agency_lang,agency_phone,agency_fare_url,agency_email\n
id_1,My agency,http://my-agency_url.com,Europe/London,EN,0123456789,http://my-agency_fare_url.com,my-mail@example.com";
    let tmp_dir = TempDir::new("navitia_model_tests").expect("create temp dir");
    let file_path = tmp_dir.path().join("agency.txt");
    let mut f = File::create(&file_path).unwrap();
    f.write_all(agency_content.as_bytes()).unwrap();

    let (networks, companies) = gtfs::read_agency(tmp_dir.path());
    tmp_dir.close().expect("delete temp dir");
    assert_eq!(1, networks.len());
    let network = networks.iter().next().unwrap().1;
    assert_eq!("id_1", network.id);
    assert_eq!(1, companies.len());
}

#[test]
#[should_panic]
fn load_2_agencies_with_no_id() {
    let agency_content = "agency_name,agency_url,agency_timezone\n
My agency 1,http://my-agency_url.com,Europe/London
My agency 2,http://my-agency_url.com,Europe/London";
    let tmp_dir = TempDir::new("navitia_model_tests").expect("create temp dir");
    let file_path = tmp_dir.path().join("agency.txt");
    let mut f = File::create(&file_path).unwrap();
    f.write_all(agency_content.as_bytes()).unwrap();
    gtfs::read_agency(tmp_dir.path());
    tmp_dir.close().expect("delete temp dir");
}


#[test]
fn load_one_stop_point() {
    let stops_content = "stop_id,stop_name,stop_lat,stop_lon\n\
    id1,my stop name,0.1,1.2";
    let tmp_dir = TempDir::new("navitia_model_tests").expect("create temp dir");
    let file_path = tmp_dir.path().join("stops.txt");
    let mut f = File::create(&file_path).unwrap();
    f.write_all(stops_content.as_bytes()).unwrap();

    let (stop_areas, stop_points) = gtfs::read_stops(tmp_dir.path());
    tmp_dir.close().expect("delete temp dir");
    assert_eq!(1, stop_areas.len());
    assert_eq!(1, stop_points.len());
    let stop_area = stop_areas.iter().next().unwrap().1;
    assert_eq!("SAid1", stop_area.id);
}
