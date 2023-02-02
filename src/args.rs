use std::{collections::HashMap, str::FromStr};

pub fn get_args() -> Result<Vec<Vec<String>>, String> {
  let parts: Vec<String> = std::env::args().collect();
  let subparts: Vec<Vec<String>> = parts[1..].chunks(2).map(|x| x.to_vec()).collect();

  if subparts.len() == 0 || parts.len() % 2 != 1 {
    // != 1 because the first argument is the path to the executable
    return Err("Invalid arguments count. Make sure there is at least one host set and even number of arguments".to_string());
  }

  return Ok(subparts);
}

fn extract_and_filter_by_option(option: String) -> Result<Vec<String>, String> {
  let args = get_args()?.into_iter().filter(|x| x.contains(&option)).map(|x| x[1].clone()).collect();

  return Ok(args);
}

pub fn extract_headers() -> Result<reqwest::header::HeaderMap, String> {
  // HashMap: HeaderName:HaderValue
  let args = extract_and_filter_by_option("-h".to_string())?;
  let mut headers = reqwest::header::HeaderMap::new();

  for arg in args {
    let (k, v) = arg.split_once(":").unwrap();
    
    headers.insert(
      reqwest::header::HeaderName::from_str(&k).unwrap(),
      reqwest::header::HeaderValue::from_str(&v).unwrap(),
    );
  }

  return Ok(headers);
}

pub fn extract_hosts() -> Result<HashMap<String, String>, String> {
  // HashMap: from:to
  let args = extract_and_filter_by_option("-u".to_string())?;
  let mut hosts: HashMap<String, String> = HashMap::new();

  if args.len() == 0 {
    return Err("No host specified. -u is necessary".to_string());
  }

  for arg in args {
    let (k, v) = arg.split_once(":").unwrap();

    hosts.insert(String::from(k), String::from(v));
  }

  return Ok(hosts);
}

pub fn get<T>(option: &str, default: T) -> T where T: FromStr {
  let arg = extract_and_filter_by_option(option.to_string());
  
  match arg {
    Ok(_) => {},
    Err(_) => { return default },
  }

  let unwrapped = arg.unwrap();

  if unwrapped.len() == 0 {
    return default;
  }
  
  return unwrapped[0].parse().unwrap_or_else(|_| default);
}
