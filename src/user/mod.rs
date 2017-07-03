use std::{cmp, fmt};

macro_rules! make_user_data {
	($($i:ident $t:tt),*,) => { make_user_data![$($i $t),*]; };
	($($i:ident $t:tt),*) => {
		#[derive(Debug)]
		pub enum Userdata {
			$($i $t),*
		}
		pub fn user_data_name(data: &Userdata) -> &str {
			match *data {
				$(Userdata::$i { .. } => stringify![$i]),*
			}
		}
	};
}

make_user_data![
];

impl cmp::PartialEq for Userdata {
	fn eq(&self, other: &Self) -> bool {
		if self as *const Userdata == other as *const Userdata {
			return true;
		}
	}
}

impl fmt::Display for Userdata {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write![f, ""]
	}
}