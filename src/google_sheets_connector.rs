//check here: https://github.com/dermesser/yup-oauth2/blob/7e26db39ca4a17f0057c2b106bcd6fd654eb986c/examples/drive_example/src/main.rs

extern crate hyper;
extern crate hyper_native_tls;
pub extern crate yup_oauth2;
extern crate google_sheets4 as sheets4;
extern crate dotenv;

use std::path::Path;
//use std::env;
//use google_sheets4::ValueRange;
//use google_sheets4::{Result, Error};
//use sheets4::Sheets;
pub use yup_oauth2::{FlowType,
    ApplicationSecret,
    read_application_secret};
use yup_oauth2::Authenticator;
use yup_oauth2::DefaultAuthenticatorDelegate;
use yup_oauth2::DiskTokenStorage;

use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

pub enum ValueOption {
    Raw, //RAW
    UserEntered, //USER_ENTERED
}

impl ValueOption {
    fn to_string(&self) -> String {
        match self {
            Self::Raw => String::from("RAW"),
            Self::UserEntered => String::from("USER_ENTERED"),
        }
    }
}

//////////GOOGLE SHEETS STRUCTS AND METHODS //////////
pub struct GoogleSheetsConnection {
	sheet_id: Option<String>,		    //url of google sheet
	tab_name: Option<String>,		    //name of tab that is being accessed
	credentials_file: Option<String>,	//file that has the connection json
}

/*
impl GoogleSheetsConnection {
    fn id_valid(&self) -> bool {
        match (*self.sheet_id, *self.tab_name, *self.credentials_file) {
            (Some(_), Some(_), Some(_)) => true,
            _ => false,
        }
    }
}
*/

pub struct GoogleSheetsRead {
	connection: GoogleSheetsConnection,	//connection info
	start_column: Option<i32>,	//start data read at this column
	start_row: Option<i32>,		//start data read at this row
	end_column: Option<i32>,		// stop reading data at this column
	end_row: Option<i32>,		// stop r&write_data.value_option}
}

pub struct GoogleSheetsWrite {
	connection: GoogleSheetsConnection,	//connection info
	write_data: sheets4::ValueRange,	//data that will be written
	value_option: Option<ValueOption>,  //value option type-RAW or USER_ENTERED
}


pub struct GoogleSheetsDelete {
	connection: GoogleSheetsConnection,	//connection info
	start_column: Option<i32>,	//start data read at this column
	start_row: Option<i32>,		//start data read at this row
	end_column: Option<i32>,		// stop reading data at this column
	end_row: Option<i32>,			// stop reading data at this row
}



// reads the provided example client secret, the quick and dirty way.
fn read_client_secret(file: String) -> yup_oauth2::ApplicationSecret {
    yup_oauth2::read_application_secret(Path::new(&file)).unwrap()
}


//google_sheets4::Sheets<hyper::client::Client,yup_oauth2::authenticator::Authenticator<yup_oauth2::authenticator_delegate::DefaultAuthenticatorDelegate, yup_oauth2::storage::DiskTokenStorage, hyper::client::Client>>

pub fn get_connection(tab:&str) -> GoogleSheetsConnection {
    dotenv::from_filename("d2_grail_obs.env").ok();
    
    let client_secret_file = dotenv::var("CREDENTIALS_FILE_NAME").unwrap();
    let google_sheet_url = dotenv::var("GOOGLE_SHEET_URL").unwrap();
    //let tab_name_test = dotenv::var("TAB_NAME_TEST").unwrap();

    let start_substr = "/spreadsheets/d/";
	let end_substr = "/edit";
    let spreadsheet_id = get_string_between(&google_sheet_url, start_substr, end_substr);

    GoogleSheetsConnection{
        sheet_id: match spreadsheet_id.len(){
            0 => None,
            _ => Some(spreadsheet_id),
        },
        tab_name: match tab.len(){
            0 => None,
            _ => Some(String::from(tab)),
        },
        credentials_file: match client_secret_file.len(){
            0 => None,
            _ => Some(client_secret_file),
        },
    }

}

pub fn get_read(conn: GoogleSheetsConnection,
	start_column: i32,
	start_row: i32,
	end_column: i32,
	end_row: i32) -> GoogleSheetsRead {
		GoogleSheetsRead{
			connection: conn,
            start_column: match start_column{
                 0 => None,
                _ => Some(start_column),
            
            },
            start_row: match start_row{
                 0 => None,
                _ => Some(start_row),
            
            },
            end_column: match end_column{
                 0 => None,
                _ => Some(end_column),
            
            },
            end_row: match end_row{ 
                 0 => None,
                _ => Some(end_row),
            
            },
        }
}

pub fn get_write(conn: GoogleSheetsConnection,
	start_column: i32,
	start_row: i32,
	write_data: Vec<Vec<String>>,
	value_option: Option<ValueOption>) -> GoogleSheetsWrite {
    let s_col = match start_column{
        0 => None,
        _ => Some(start_column),
    };
    
    let s_row = match start_row{
        0 => None,
        _ => Some(start_row),
    };
    
    let (num_row, num_col) = calc_range_from_vec_vec(&write_data);

    let e_col = match start_column+num_col{
        0 => None,
        _ => Some(start_column+num_col),
    };

    let e_row = match start_row+num_row{
        0 => None,
        _ => Some(start_row+num_row),
    };
   
    let conn_tab  = &conn.tab_name.unwrap();
    let tab = &conn_tab;
    let write_range = to_a1_notation(tab,
        s_col,
        s_row,
        e_col,
        e_row);

    let val_range = sheets4::ValueRange{
		range: Some(write_range),
		values: Some(write_data),
		major_dimension: Some(String::from("ROWS")),
	};


    GoogleSheetsWrite{
        connection: conn,
        write_data: val_range,
		value_option: Some(ValueOption::Raw),
	}
}

pub fn get_delete(conn: GoogleSheetsConnection,
                  start_column: i32,
                  start_row: i32,
                  end_column: i32,
                  end_row: i32) -> GoogleSheetsDelete{
    let s_col = match start_column{
        0 => None,
        _ => Some(start_column),
    };
    
    let s_row = match start_row{
        0 => None,
        _ => Some(start_row),
    };

    let e_col = match end_column{
        0 => None,
        _ => Some(end_column),
    };

    let e_row = match end_row{
        0 => None,
        _ => Some(end_row),
    };

    GoogleSheetsDelete{
        connection: conn,
        start_column: s_col,
        start_row: s_row,
        end_column: e_col,
        end_row: e_row,
    }
}

fn get_hub(conn: GoogleSheetsConnection) -> sheets4::Sheets<
    hyper::client::Client,
    yup_oauth2::Authenticator<
        yup_oauth2::DefaultAuthenticatorDelegate,
        yup_oauth2::DiskTokenStorage,
        hyper::client::Client>> {
	let sheet_id = conn.sheet_id;
    let secret = read_client_secret(conn.credentials_file.unwrap());
	let client = hyper::Client::with_connector(
        hyper::net::HttpsConnector::new(hyper_native_tls::NativeTlsClient::new().unwrap()));
	
	let authenticator = Authenticator::new(&secret,
		yup_oauth2::DefaultAuthenticatorDelegate,
		client,
		yup_oauth2::DiskTokenStorage::new(&"token_store.json".to_string())
			.unwrap(),
		Some(FlowType::InstalledInteractive));

	let client = hyper::Client::with_connector(
		hyper::net::HttpsConnector::new(hyper_native_tls::NativeTlsClient::new().unwrap()));
	
	//return the authenticated "hub"
	sheets4::Sheets::new(client, authenticator)
}


//gsc_reader returns an array of data based on the sheet, tab, and range provided.  Use 0 for endRow to return all rows
pub fn gsc_reader(read_data: GoogleSheetsRead) -> Vec<Vec<String>> {
    let read_range = to_a1_notation(&read_data.connection.tab_name.unwrap(),
        read_data.start_column,
        read_data.start_row,
        read_data.end_column,
        read_data.end_row);
    println!("Read Range: {}", read_range);

    let hub = get_hub(read_data.connection);

    let result = hub.spreadsheets().values_get(&read_data.connection.sheet_id.unwrap(), &read_range)
                .major_dimension("ROWS")
                .doit();

    match result {
        Err(e) => {
            println!("{} -- returning blank", e);
            let vec_str =vec![String::from("")];
            let vec_vec = vec![vec_str];
            vec_vec
        },
        Ok(res) => res.1.values.unwrap(),
    }
}

//gsc_writer takes a sheet struct and writes it into the sheet. Returns true on success and false on failure.
pub fn gsc_writer(write_data: GoogleSheetsWrite) /*TODO: Return type */{
    let write_range = write_data.write_data.range.unwrap();
    println!("Write Range: {}", write_range);
    let hub = get_hub(write_data.connection);
	
	let req = write_data.write_data; 


    //result retuns (Response, ValueRange) tuple
	let result = hub.spreadsheets().values_update(req,
                &write_data.connection.sheet_id.unwrap(),
                &write_range)
				.value_input_option(&write_data.value_option.unwrap().to_string())
                //.major_dimension("ROWS")
				.doit();
	
	println!("Result: {:#?}", &result);
}


/*
    let client_secret_file = dotenv::var("CREDENTIALS_FILE_NAME").unwrap();
    let google_sheet_url = dotenv::var("GOOGLE_SHEET_URL").unwrap();
     let tab_name_test = dotenv::var("TAB_NAME_TEST").unwrap();

    let start_substr = "/spreadsheets/d/";
	let end_substr = "/edit";
    let spreadsheet_id = get_string_between(&google_sheet_url, start_substr, end_substr);

	let secret = read_client_secret(client_secret_file.to_string());

	let client = hyper::Client::with_connector(
        HttpsConnector::new(NativeTlsClient::new().unwrap()));
	
	let authenticator = Authenticator::new(&secret,
		DefaultAuthenticatorDelegate,
		client,
		DiskTokenStorage::new(&"token_store.json".to_string())
			.unwrap(),
		Some(FlowType::InstalledInteractive));

	let client = hyper::Client::with_connector(
		HttpsConnector::new(NativeTlsClient::new().unwrap()));
	
	//return the authenticated "hub"
    let hub = Sheets::new(client, authenticator);
        //println!("{}", lib::type_of(&hub));
    
    let mut range = "TestSheet".to_string();
    
    range.push_str("!A1:B");

    //result retuns (Response, ValueRange) tuple
    let result = hub.spreadsheets().values_get(&spreadsheet_id, &range)
                .major_dimension("ROWS")
                .doit();

    //handle errors
    match result {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
             Error::HttpError(_)
            |Error::MissingAPIKey
            |Error::MissingToken(_)
            |Error::Cancelled
            |Error::UploadSizeLimitExceeded(_, _)
            |Error::Failure(_)
            |Error::BadRequest(_)
            |Error::FieldClash(_)
            |Error::JsonDecodeError(_, _) => {
                println!("{} -- returning blank", e);
                let vec_str =vec![String::from("")];
                let vec_vec = vec![vec_str];
                vec_vec
            }
        },
        //Ok(res) effectively "unwraps" the ok option of res
        //res.1 to access the ValueRange part of the tuple
        //res.1.values to access the values option
        //res.1.values.unwrap() to unwrap the actual values
        //this is a Vec<Vec<String>> type,
        //a vector of a vector of strings
        Ok(res) => res.1.values.unwrap(),
    }
}
*/

/*


//GSDeleter takes a sheet struct and writes it into the sheet. Returns true on success and false on failure.
func GSDeleter(gscdata types.GSCDelete) bool {
	//fmt.Println(binary.Size(b))
	startSubstr := "/spreadsheets/d/"
	endSubstr := "/edit"
	spreadsheetID := lib.GetStringBetween(gscdata.SheetURL, startSubstr, endSubstr)

	deleteRange := gscdata.TabName + "!"
	deleteRange = deleteRange + lib.IntToCharStrArr(gscdata.StartColumn) + strconv.Itoa(gscdata.StartRow)
	deleteRange = deleteRange + ":" + lib.IntToCharStrArr(gscdata.EndColumn) + strconv.Itoa(gscdata.EndRow)
	fmt.Println(deleteRange)

	// If modifying these scopes, delete your previously saved token.json.
	config, err := google.ConfigFromJSON(gscdata.B, "https://www.googleapis.com/auth/spreadsheets")
	if err != nil {
		log.Fatalf("Unable to parse client secret file to config: %v", err)
	}
	client := getClient(config)

	srv, err := sheets.New(client)
	if err != nil {
		log.Fatalf("Unable to retrieve Sheets client: %v", err)
	}

	requestBody := &sheets.ClearValuesRequest{} // requestBody is empty for delete/clear call

	resp, err := srv.Spreadsheets.Values.Clear(spreadsheetID, deleteRange, requestBody).Do()
	//.Context(ctx).Do()
	if err != nil {
		log.Fatalf("Unable to write data to sheet: %v", err)
		return false
	}
	log.Printf("Delete Response: %v", resp)
	return true
}
*/


//get_string_between returns empty string if no start string found
fn get_string_between(full_str: &str , start: &str, end: &str) -> String {    
    let start_i = full_str.find(start).unwrap()+start.len();
	if start_i < 1 {
		String::from("")
	} else {
        let end_i = full_str.find(end).unwrap();
        String::from(&full_str[start_i..end_i])
    }
}

//int_to_char_string returns the alpha variant of a numeric column...allows for far more options than currently
//available in google sheets
fn int_to_char_string(i: i32) -> String {
	 let string_arr = vec![".", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
		"N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "AA", "AB", "AC", "AD",
		"AE", "AF", "AG", "AH", "AI", "AJ", "AK", "AL", "AM", "AN", "AO", "AP", "AQ", "AR", "AS",
		"AT", "AU", "AV", "AW", "AX", "AY", "AZ", "BA", "BB", "BC", "BD", "BE", "BF", "BG", "BH",
		"BI", "BJ", "BK", "BL", "BM", "BN", "BO", "BP", "BQ", "BR", "BS", "BT", "BU", "BV", "BW",
		"BX", "BY", "BZ", "CA", "CB", "CC", "CD", "CE", "CF", "CG", "CH", "CI", "CJ", "CK", "CL",
		"CM", "CN", "CO", "CP", "CQ", "CR", "CS", "CT", "CU", "CV", "CW", "CX", "CY", "CZ", "DA",
		"DB", "DC", "DD", "DE", "DF", "DG", "DH", "DI", "DJ", "DK", "DL", "DM", "DN", "DO", "DP", 
		"DQ", "DR", "DS", "DT", "DU", "DV", "DW", "DX", "DY", "DZ", "EA", "EB", "EC", "ED", "EE",
		"EF", "EG", "EH", "EI", "EJ", "EK", "EL", "EM", "EN", "EO", "EP", "EQ", "ER", "ES", "ET",
		"EU", "EV", "EW", "EX", "EY", "EZ", "FA", "FB", "FC", "FD", "FE", "FF", "FG", "FH", "FI",
		"FJ", "FK", "FL", "FM", "FN", "FO", "FP", "FQ", "FR", "FS", "FT", "FU", "FV", "FW", "FX",
		"FY", "FZ", "GA", "GB", "GC", "GD", "GE", "GF", "GG", "GH", "GI", "GJ", "GK", "GL", "GM",
		"GN", "GO", "GP", "GQ", "GR", "GS", "GT", "GU", "GV", "GW", "GX", "GY", "GZ", "HA", "HB",
		"HC", "HD", "HE", "HF", "HG", "HH", "HI", "HJ", "HK", "HL", "HM", "HN", "HO", "HP", "HQ",
		"HR", "HS", "HT", "HU", "HV", "HW", "HX", "HY", "HZ", "IA", "IB", "IC", "ID", "IE", "IF",
		"IG", "IH", "II", "IJ", "IK", "IL", "IM", "IN", "IO", "IP", "IQ", "IR", "IS", "IT", "IU",
		"IV", "IW", "IX", "IY", "IZ", "JA", "JB", "JC", "JD", "JE", "JF", "JG", "JH", "JI", "JJ",
		"JK", "JL", "JM", "JN", "JO", "JP", "JQ", "JR", "JS", "JT", "JU", "JV", "JW", "JX", "JY",
		"JZ", "KA", "KB", "KC", "KD", "KE", "KF", "KG", "KH", "KI", "KJ", "KK", "KL", "KM", "KN",
		"KO", "KP", "KQ", "KR", "KS", "KT", "KU", "KV", "KW", "KX", "KY", "KZ", "LA", "LB", "LC",
		"LD", "LE", "LF", "LG", "LH", "LI", "LJ", "LK", "LL", "LM", "LN", "LO", "LP", "LQ", "LR",
		"LS", "LT", "LU", "LV", "LW", "LX", "LY", "LZ", "MA", "MB", "MC", "MD", "ME", "MF", "MG",
		"MH", "MI", "MJ", "MK", "ML", "MM", "MN", "MO", "MP", "MQ", "MR", "MS", "MT", "MU", "MV",
		"MW", "MX", "MY", "MZ", "NA", "NB", "NC", "ND", "NE", "NF", "NG", "NH", "NI", "NJ", "NK",
		"NL", "NM", "NN", "NO", "NP", "NQ", "NR", "NS", "NT", "NU", "NV", "NW", "NX", "NY", "NZ",
		"OA", "OB", "OC", "OD", "OE", "OF", "OG", "OH", "OI", "OJ", "OK", "OL", "OM", "ON", "OO",
		"OP", "OQ", "OR", "OS", "OT", "OU", "OV", "OW", "OX", "OY", "OZ", "PA", "PB", "PC", "PD",
		"PE", "PF", "PG", "PH", "PI", "PJ", "PK", "PL", "PM", "PN", "PO", "PP", "PQ", "PR", "PS",
		"PT", "PU", "PV", "PW", "PX", "PY", "PZ", "QA", "QB", "QC", "QD", "QE", "QF", "QG", "QH",
		"QI", "QJ", "QK", "QL", "QM", "QN", "QO", "QP", "QQ", "QR", "QS", "QT", "QU", "QV", "QW",
		"QX", "QY", "QZ", "RA", "RB", "RC", "RD", "RE", "RF", "RG", "RH", "RI", "RJ", "RK", "RL",
		"RM", "RN", "RO", "RP", "RQ", "RR", "RS", "RT", "RU", "RV", "RW", "RX", "RY", "RZ", "SA",
		"SB", "SC", "SD", "SE", "SF", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SP",
		"SQ", "SR", "SS", "ST", "SU", "SV", "SW", "SX", "SY", "SZ", "TA", "TB", "TC", "TD", "TE",
		"TF", "TG", "TH", "TI", "TJ", "TK", "TL", "TM", "TN", "TO", "TP", "TQ", "TR", "TS", "TT",
		"TU", "TV", "TW", "TX", "TY", "TZ", "UA", "UB", "UC", "UD", "UE", "UF", "UG", "UH", "UI",
		"UJ", "UK", "UL", "UM", "UN", "UO", "UP", "UQ", "UR", "US", "UT", "UU", "UV", "UW", "UX",
		"UY", "UZ", "VA", "VB", "VC", "VD", "VE", "VF", "VG", "VH", "VI", "VJ", "VK", "VL", "VM",
		"VN", "VO", "VP", "VQ", "VR", "VS", "VT", "VU", "VV", "VW", "VX", "VY", "VZ", "WA", "WB",
		"WC", "WD", "WE", "WF", "WG", "WH", "WI", "WJ", "WK", "WL", "WM", "WN", "WO", "WP", "WQ",
		"WR", "WS", "WT", "WU", "WV", "WW", "WX", "WY", "WZ", "XA", "XB", "XC", "XD", "XE", "XF",
		"XG", "XH", "XI", "XJ", "XK", "XL", "XM", "XN", "XO", "XP", "XQ", "XR", "XS", "XT", "XU",
		"XV", "XW", "XX", "XY", "XZ", "YA", "YB", "YC", "YD", "YE", "YF", "YG", "YH", "YI", "YJ",
		"YK", "YL", "YM", "YN", "YO", "YP", "YQ", "YR", "YS", "YT", "YU", "YV", "YW", "YX", "YY",
		"YZ", "ZA", "ZB", "ZC", "ZD", "ZE", "ZF", "ZG", "ZH", "ZI", "ZJ", "ZK", "ZL", "ZM", "ZN",
		"ZO", "ZP", "ZQ", "ZR", "ZS", "ZT", "ZU", "ZV", "ZW", "ZX", "ZY", "ZZ"];

	let vec_len:i32 = string_arr.len() as i32 - 1;
	if 1 <= i && i <= vec_len {
		string_arr[i as usize].to_string()
	} else {
		"ERROR".to_string()
	}
}


// calc_range_from_vec_vec takes a vector of vector of strings and returns the number of rows and columns (as a tuple)
pub fn calc_range_from_vec_vec(range: &[Vec<String>]) -> (i32, i32) {
	//(rows, cols)
    (range.len() as i32, range[0].len() as i32)
}


//to_a1_notation takes a numeric column and row representation and changes
//it to a1 notation.
pub fn to_a1_notation(tab: &str, start_column: Option<i32>,
	start_row: Option<i32>, end_column: Option<i32>, end_row: Option<i32>) -> String {
	let mut a1_notation = String::from("'");
	a1_notation.push_str(tab);
	a1_notation.push_str("'");
	match (start_column, start_row, end_column, end_row){
		(None, None, None, None) => a1_notation,
		(Some(s_col), Some(s_row), Some(e_col), None) => {
				a1_notation.push_str("!");
				a1_notation.push_str(&int_to_char_string(s_col));
				a1_notation.push_str(&s_row.to_string());
				a1_notation.push_str(":");
				a1_notation.push_str(&int_to_char_string(e_col));
				a1_notation
			},
		(Some(s_col), Some(s_row), Some(e_col), Some(e_row)) => {
				a1_notation.push_str("!");
				a1_notation.push_str(&int_to_char_string(s_col));
				a1_notation.push_str(&s_row.to_string());
				a1_notation.push_str(":");
				a1_notation.push_str(&int_to_char_string(e_col));
				a1_notation.push_str(&e_row.to_string());
				a1_notation
			},
		_ => {
				println!("{}", "Failed to parse to A1 Notation, returning empty string");
				println!("{:#?}", (tab, start_column, start_row, end_column, end_row));
				String::from("")
			}
	}
}
