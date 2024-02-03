use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use std::process::Command;
use std::fs::File;
use std::io::Write;
use bytes::Bytes;


fn metadata_template(visible_name: &str) -> String {
  let start = SystemTime::now();
  let since_the_epoch = start
      .duration_since(UNIX_EPOCH).unwrap();
  let milliseconds = since_the_epoch.as_millis();

  format!("{{
      \"deleted\": false,
      \"lastModified\": \"{milliseconds}\",
      \"lastOpened\": \"0\",
      \"lastOpenedPage\": 0,
      \"metadatamodified\": true,
      \"modified\": true,
      \"parent\": \"\",
      \"pinned\": false,
      \"synced\": false,
      \"type\": \"DocumentType\",
      \"version\": 0,
      \"visibleName\": \"{visible_name}\"
  }}")
}

const CONTENT_TEMPLATE: &str = "{
  \"fileType\": \"pdf\"  
}";

pub fn store_pdf(bytes: Bytes, job_name: &str) {
  let base_path = "/home/root/.local/share/remarkable/xochitl/";
  let uuid = Uuid::new_v4();
  let uuid = uuid.as_hyphenated();
  let path = format!("{base_path}{uuid}.pdf");
  let mut file = File::create(path).unwrap();
  file.write_all(&bytes).unwrap();

  let metadata = metadata_template(job_name);
  let path = format!("{base_path}{uuid}.metadata");
  let mut file = File::create(path).unwrap();
  file.write_all(metadata.as_bytes()).unwrap();

  let path = format!("{base_path}{uuid}.content");
  let mut file = File::create(path).unwrap();
  file.write_all(CONTENT_TEMPLATE.as_bytes()).unwrap();

  Command::new("systemctl")
      .args(["restart", "xochitl"])
      .output().unwrap();
}