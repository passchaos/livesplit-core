extern crate livesplit_core;

mod save {
    use std::fs::File;
    use std::io::BufReader;
    use livesplit_core::Run;
    use livesplit_core::run::parser::livesplit;
    use livesplit_core::run::saver::{file_write, splitty, urn, wsplit};

    fn file(path: &str) -> BufReader<File> {
        BufReader::new(File::open(path).unwrap())
    }

    fn livesplit(path: &str) -> Run {
        livesplit::parse(file(path), None).unwrap()
    }

    #[test]
    fn wsplit() {
        let run = livesplit("tests/run_files/livesplit1.6_gametime.lss");
        let mut vec = Vec::new();
        wsplit::save(&run, file_write::OnlyStoreMainFile(&mut vec)).unwrap();
        let got = String::from_utf8(vec).unwrap();
        let expected = r#"Title=Dishonored - Any%
Attempts=1958
Offset=0
Size=152,25
WatchYr<3Die,0,191.265,5.432
Elevator Land,0,264.86,2.003
Gross, Rats,0,395.925,131.38
Try Not Falling?,0,628.423,231.735
Men of the Faith,0,775.708,146.673
Fuckin Celia,0,894.514,112.506
Tr4p Haus,0,1066.217,165.834
Troubled Waters,0,1371.376,160.397
RNG Roulette,0,1586.956,210.858
Not At Fault,0,1833.368,236.252
Amateur Work,0,2211.204,378.561
Don't Mind Me,0,2278.29,61.884
H9 U Havelock,0,2401.517,122.007
Icons="","","","","","","","","","","","",""
"#;
        assert_eq!(expected, got);
    }

    #[test]
    fn urn() {
        let mut run = livesplit("tests/run_files/livesplit1.6_gametime.lss");
        let mut vec = Vec::new();
        urn::save_with_recalculation(&mut run, &mut vec).unwrap();
        let got = String::from_utf8(vec).unwrap();
        let expected = r#"{"title":"Dishonored - Any%","attempt_count":1958,"splits":[{"title":"WatchYr<3Die","time":"00:03:11.2650000","best_time":"00:00:05.4320000","best_segment":"00:00:05.4320000"},{"title":"Elevator Land","time":"00:04:24.8600000","best_time":"00:04:21.9200000","best_segment":"00:00:02.0030000"},{"title":"Gross, Rats","time":"00:06:35.9250000","best_time":"00:06:34.5419990","best_segment":"00:02:11.3800000"},{"title":"Try Not Falling?","time":"00:10:28.4230000","best_time":"00:10:27.3159990","best_segment":"00:03:51.7350000"},{"title":"Men of the Faith","time":"00:12:55.7080000","best_time":"00:12:55.7080000","best_segment":"00:02:26.6730000"},{"title":"Fuckin Celia","time":"00:14:54.5140000","best_time":"00:14:54.5140000","best_segment":"00:01:52.5060000"},{"title":"Tr4p Haus","time":"00:17:46.2170000","best_time":"00:17:46.2170000","best_segment":"00:02:45.8340000"},{"title":"Troubled Waters","time":"00:22:51.3760000","best_time":"00:22:51.3760000","best_segment":"00:02:40.3970000"},{"title":"RNG Roulette","time":"00:26:26.9560000","best_time":"00:26:26.9560000","best_segment":"00:03:30.8580000"},{"title":"Not At Fault","time":"00:30:33.3680000","best_time":"00:30:33.3680000","best_segment":"00:03:56.2520000"},{"title":"Amateur Work","time":"00:36:51.2040000","best_time":"00:36:51.2040000","best_segment":"00:06:18.5610000"},{"title":"Don't Mind Me","time":"00:37:58.2900000","best_time":"00:37:58.2900000","best_segment":"00:01:01.8840000"},{"title":"H9 U Havelock","time":"00:40:01.5170000","best_time":"00:40:01.5170000","best_segment":"00:02:02.0070000"}]}"#;
        assert_eq!(expected, got);
    }

    #[test]
    fn splitty() {
        let run = livesplit("tests/run_files/livesplit1.6_gametime.lss");
        let mut vec = Vec::new();
        splitty::save(&run, &mut vec).unwrap();
        let got = String::from_utf8(vec).unwrap();
        let expected = r#"{"timer_name":"Dishonored - Any%","run_name":"Dishonored - Any%","hash":"","start_delay":-0.0,"run_count":1958,"splits":[{"name":"WatchYr<3Die","pb_split":191265.0,"pb_duration":191265.0,"split_best":5432.0},{"name":"Elevator Land","pb_split":264860.0,"pb_duration":73595.0,"split_best":2003.0},{"name":"Gross, Rats","pb_split":395925.0,"pb_duration":131065.0,"split_best":131380.0},{"name":"Try Not Falling?","pb_split":628423.0,"pb_duration":232498.0,"split_best":231735.0},{"name":"Men of the Faith","pb_split":775708.0,"pb_duration":147285.0,"split_best":146673.0},{"name":"Fuckin Celia","pb_split":894514.0,"pb_duration":118806.0,"split_best":112506.0},{"name":"Tr4p Haus","pb_split":1066217.0,"pb_duration":171703.0,"split_best":165834.0},{"name":"Troubled Waters","pb_split":1371376.0,"pb_duration":305159.0,"split_best":160397.0},{"name":"RNG Roulette","pb_split":1586956.0,"pb_duration":215580.0,"split_best":210858.0},{"name":"Not At Fault","pb_split":1833368.0,"pb_duration":246412.0,"split_best":236252.0},{"name":"Amateur Work","pb_split":2211204.0,"pb_duration":377836.0,"split_best":378561.0},{"name":"Don't Mind Me","pb_split":2278290.0,"pb_duration":67086.0,"split_best":61884.0},{"name":"H9 U Havelock","pb_split":2401517.0,"pb_duration":123227.0,"split_best":122007.0}],"timer_type":0}"#;
        assert_eq!(expected, got);
    }
}
