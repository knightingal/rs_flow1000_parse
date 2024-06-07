
pub struct DesignationData {
  pub char_len: u8,
  state: DesignationState,
  pub num_len: u8,
  pub char_part: String,
  pub num_part: String,
  pub char_final: Option<String>,
  pub num_final: Option<String>,
}

impl DesignationData {
  pub fn reset(&mut self) {
    self.char_len = 0;
    self.num_len = 0;
    self.state = DesignationState::Init;
    self.char_part.clear();
    self.num_part.clear();
  }
}

enum DesignationState {
  Init,
  Char,
  Num,
  Split,
  End,
}

enum DesignationTranc {
  Num,
  Char,
  Split,
  Other,
}

fn state_end(designation_state: &mut DesignationData) {
  designation_state.state = DesignationState::End;
  if designation_state.char_final.is_none() {
    let mut char_final = designation_state.char_part.clone();
    char_final = char_final.to_uppercase();
    designation_state.char_final = Option::Some(char_final);
  }
  if designation_state.num_final.is_none() {
    let mut num_final = designation_state.num_part.clone();
    while num_final.len() > 3 && num_final.starts_with("0") {
      num_final = num_final.split_at(1).1.to_string();
    }  
    designation_state.num_final = Option::Some(num_final);
  }
}

fn state_trans(ch: &char, designation_state: &mut DesignationData, tranc_code: DesignationTranc) {
  match designation_state.state {
    DesignationState::Init => {
      match tranc_code {
        DesignationTranc::Char => {
          designation_state.state = DesignationState::Char;
          designation_state.char_len = designation_state.char_len + 1;
          designation_state.char_part.push(*ch);
        },
        _ => {}
      }
    },
    DesignationState::Char => {
      match tranc_code {
        DesignationTranc::Num => {
            designation_state.state = DesignationState::Num;
            designation_state.num_len = designation_state.num_len + 1;
            designation_state.num_part.push(*ch);
        }
        DesignationTranc::Split => {
          designation_state.state = DesignationState::Split;
        }
        DesignationTranc::Char => {
          if designation_state.char_len == CHAR_MAX {
            designation_state.reset();
          } else {
            designation_state.state = DesignationState::Char;
            designation_state.char_len = designation_state.char_len + 1;
            designation_state.char_part.push(*ch);
          }
        }
        _ => {designation_state.reset();}  
      }

    },
    DesignationState::Num => {
      match tranc_code {
        DesignationTranc::Num => {
          if designation_state.num_len == NUM_MAX {
            designation_state.reset();
          } else {
            designation_state.state = DesignationState::Num;
            designation_state.num_len = designation_state.num_len + 1;
            designation_state.num_part.push(*ch);
          }
        }
        _ => {
          if designation_state.num_len >= NUM_MIN {

            let mut char_final = designation_state.char_part.clone();
            char_final = char_final.to_uppercase();
            let mut num_final = designation_state.num_part.clone();
            while num_final.len() > 3 && num_final.starts_with("0") {
              num_final = num_final.split_at(1).1.to_string();
            }  
            let mut whole_designation = String::from(&char_final);
            whole_designation.push_str(num_final.as_str());

            if whole_designation == "H264" || whole_designation == "H265" {
              return;
            }

            designation_state.char_final = Option::Some(char_final);
            designation_state.num_final = Option::Some(num_final);
          }
          designation_state.reset();
        }  
      }

    },
    DesignationState::Split => {
      match tranc_code {
        DesignationTranc::Num => {
            designation_state.state = DesignationState::Num;
            designation_state.num_len = designation_state.num_len + 1;
            designation_state.num_part.push(*ch);
        },
        DesignationTranc::Char => {
          designation_state.reset();
            designation_state.state = DesignationState::Char;
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
const CHAR_MAX: u8 = 6;

pub fn parse_designation(file_name: &String) -> DesignationData {
  let chars = file_name.chars();
  let mut designation_state: DesignationData = DesignationData { 
    char_len: (0), 
    state: (DesignationState::Init), 
    num_len: (0), 
    char_part: String::new(), 
    num_part: String::new(),
    num_final: Option::None,
    char_final: Option::None,

  };
  for char_it in chars {
    if char_it.is_ascii_alphabetic() {
      state_trans(&char_it, &mut designation_state, DesignationTranc::Char);
    } else if char_it.is_ascii_digit() {
      state_trans(&char_it, &mut designation_state, DesignationTranc::Num);
    } else if char_it == '-' {
      state_trans(&char_it, &mut designation_state, DesignationTranc::Split);
    } else {
      state_trans(&char_it, &mut designation_state, DesignationTranc::Other);
    }
  }
  state_end(&mut designation_state);

  return designation_state;
}