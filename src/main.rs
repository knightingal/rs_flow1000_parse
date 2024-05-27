use axum::{extract::Path, routing::{get, post}, Json, Router};
use handles::{mount_config_handler, mp4_dir_handler, mp4_dir_handler1, video_detail, video_info_handler, POOL};
use hyper::StatusCode;
use mysql::Pool;
use serde_derive::{Deserialize, Serialize};

mod test_main;
mod test_aes;
mod handles;
mod test_designation;


#[tokio::main]
async fn main() {

  let url = "mysql://root:000000@localhost:3306/mp4viewer";
  let pool = Pool::new(url).unwrap();
  let box_pool = Box::new(Pool::new(url).unwrap());
  unsafe {
    POOL = Some(Box::leak(box_pool))
  }

  let app = Router::new()
    .route("/", get(root))
    .route("/users/name/:name/age/:age", post(create_user))
    .route("/video-info/:base_index/*sub_dir", get(video_info_handler))
    .route("/mount-config", get(mount_config_handler))

    .route("/mp4-dir/:base_index/", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index/*sub_dir", get(mp4_dir_handler))

    .route("/video-detail/:id", get(video_detail))
    .with_state(pool)
    ;
  let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
  "Hello World!"
}


async fn create_user(Path((name,age)): Path<(String, u32)>, Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
  let name:String = name;
  let age: u32 = age;

  let user = User {
    id: 1337,
    age, name,
    username: payload.username
  };

  (StatusCode::CREATED, Json(user))
}

fn is_charact(byte:&u8) -> bool {
  byte >= &b'a' && byte <= &b'z' || byte >= &b'A' && byte <= &b'Z'
}

fn is_number(byte:&u8) -> bool {
  byte >= &b'0' && byte <= &b'9' 
}

struct DesignationData {
  char_len: u8,
  state: DesignationState,
  num_len: u8,
  char_part: String,
  num_part: String,
  char_final: Option<String>,
  num_final: Option<String>,
}

impl DesignationData {
  pub fn reset(&mut self) {
    self.char_len = 0;
    self.num_len = 0;
    self.state = DesignationState::init;
    self.char_part.clear();
    self.num_part.clear();
  }
}

enum DesignationState {
  init,
  char,
  num,
  split,
  end,
}

enum DesignationTranc {
  num,
  char,
  split,
  other,
}

fn state_end(designation_state: &mut DesignationData) {
  designation_state.state = DesignationState::end;
  if designation_state.char_final.is_none() {
    designation_state.char_final = Option::Some(String::from(designation_state.char_part.as_str()));
  }
  if designation_state.num_final.is_none() {
    designation_state.num_final = Option::Some(String::from(designation_state.num_part.as_str()));
  }
}

fn state_trans(ch: &char, designation_state: &mut DesignationData, tranc_code: DesignationTranc) {
  match designation_state.state {
    DesignationState::init => {
      match tranc_code {
        DesignationTranc::char => {
          designation_state.state = DesignationState::char;
          designation_state.char_len = designation_state.char_len + 1;
          designation_state.char_part.push(*ch);
        },
        _ => {}
      }
    },
    DesignationState::char => {
      match tranc_code {
        DesignationTranc::num => {
            designation_state.state = DesignationState::num;
            designation_state.num_len = designation_state.num_len + 1;
            designation_state.num_part.push(*ch);
        }
        DesignationTranc::split => {
          designation_state.state = DesignationState::split;
        }
        DesignationTranc::char => {
          if designation_state.char_len == 4 {
            designation_state.reset();
          } else {
            designation_state.state = DesignationState::char;
            designation_state.char_len = designation_state.char_len + 1;
            designation_state.char_part.push(*ch);
          }
        }
        _ => {designation_state.reset();}  
      }

    },
    DesignationState::num => {
      match tranc_code {
        DesignationTranc::num => {
          if designation_state.num_len == NUM_MAX {
            designation_state.reset();
          } else {
            designation_state.state = DesignationState::num;
            designation_state.num_len = designation_state.num_len + 1;
            designation_state.num_part.push(*ch);
          }
        }
        _ => {
          if designation_state.num_len >= NUM_MIN {
            designation_state.char_final = Option::Some(String::from(designation_state.char_part.as_str()));
            designation_state.num_final = Option::Some(String::from(designation_state.num_part.as_str()));
          }
          designation_state.reset();
        }  
      }

    },
    DesignationState::split => {
      match tranc_code {
        DesignationTranc::num => {
            designation_state.state = DesignationState::num;
            designation_state.num_len = designation_state.num_len + 1;
            designation_state.num_part.push(*ch);
        },
        DesignationTranc::char => {
          designation_state.reset();
            designation_state.state = DesignationState::char;
            designation_state.char_len = designation_state.char_len + 1;
            designation_state.char_part.push(*ch);
        },
        _ => {
          designation_state.reset();
        },
      }
    },
    _ => {
      designation_state.reset();
    }

      
  }

}

const NUM_MAX: u8 = 6;
const NUM_MIN: u8 = 3;

fn parse_designation(file_name: &String) -> DesignationData {
  let chars = file_name.chars();
  let mut designation_state: DesignationData = DesignationData { 
    char_len: (0), 
    state: (DesignationState::init), 
    num_len: (0), 
    char_part: String::new(), 
    num_part: String::new(),
    num_final: Option::None,
    char_final: Option::None,

  };
  for char_it in chars {
    if char_it.is_ascii_alphabetic() {
      state_trans(&char_it, &mut designation_state, DesignationTranc::char);
    } else if char_it.is_ascii_digit() {
      state_trans(&char_it, &mut designation_state, DesignationTranc::num);
    } else if char_it == '-' {
      state_trans(&char_it, &mut designation_state, DesignationTranc::split);
    } else {
      state_trans(&char_it, &mut designation_state, DesignationTranc::other);
    }
  }
  state_end(&mut designation_state);

  return designation_state;
}

fn sample_code_list() -> Vec<&'static str> {
  return vec![
    "MIDD",
    "MDED",
    "MIRD",
    "MIGD",
    "MIID",
    "MIAD",
    "MIBD",
    "MIMK",
    "ASS",
    "ES",
    "NEW",
    "REPLAY",
    "LEGEND",
    "MINT",
    "ONED",
    "SOE",
    "SPS",
    "ONSD",
    "KIRD",
    "BLK",
    "KISD",
    "GG",
    "JJ",
    "KK",
    "SCOP",
    "TBL",
    "MZQ",
    "YSN",
    "DXMN",
    "LABS",
    "AM",
    "BF",
    "SUPD",
    "NSS",
    "ATOM",
    "BDD",
    "ARSO",
    "FAA",
    "SW",
    "NGD",
    "TBL",
    "HBAD",
    "TMDI",
    "DCS",
    "CWM",
    "OKAD",
    "MVBD",
    "MVSD",
    "SUNS",
    "UMD",
    "MOMJ",
    "TARD",
    "HUNT",
    "DVDES",
    "ROY",
    "SASS",
    "OLS",
    "ATT",
    "INF",
    "DCM",
    "MN",
    "AGEMIX",
    "BDSR",
    "WDI",
    "WSS",
    "NATR",
    "MAST",
    "ONCE",
    "WOBB",
    "ODFR",
    "ODFW",
    "APAD",
    "APAR",
    "SERO",
    "DXN",
    "HUNT",
    "GAR",
    "SVDVD",
    "RCT",
    "NGKS",
    "RD",
    "KUF",
    "IPTD",
    "IPZIPZ",
    "IDBD",
    "SUPD",
    "IPSD",
    "SVND",
    "HBAD",
    "MV",
    "VSPDS",
    "VSPDR",
    "FSET",
    "DANDY",
    "LADY",
    "SVDVD",
    "NMD",
    "UFD",
    "CXD",
    "BBI",
    "BEB",
    "NST",
    "BUR",
    "FTA",
    "NEO",
    "CRPD",
    "JUKD",
    "JUC",
    "ATID",
    "RBD",
    "JBD",
    "SHKD",
    "SSPD",
    "MDYD",
    "PGD",
    "PJD",
    "WANZ",
    "KAWD",
    "KAPD",
    "MXGS",
    "MX3DS",
    "MXSPS",
    "DDT",
    "STAR",
    "SACE",
    "SDDM",
    "SDDE",
    "SDMT",
    "OVDES",
    "NHDTA",
    "IESP",
    "IDOL",
    "IENE",
    "OPEN",
    "FSDSS",
  ];
}

#[derive(Deserialize)]
struct CreateUser {
  username: String,
}

#[derive(Serialize)]
struct User {
  id: u64,
  age: u32,
  name: String,
  username: String,
}
