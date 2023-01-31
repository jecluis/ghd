// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use directories::BaseDirs;
use std::path::PathBuf;

pub struct Paths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub db_path: PathBuf,
}

impl Default for Paths {
    fn default() -> Self {
        let basedirs = BaseDirs::new().expect("unable to obtain base dirs");
        let datadir = basedirs.data_local_dir().join("ghd");
        let confdir = basedirs.config_dir().join("ghd");
        let dbpath = PathBuf::new().join(&datadir).join("ghd.sqlite3");

        Paths {
            data_dir: datadir,
            config_dir: confdir,
            db_path: dbpath,
        }
    }
}

impl Paths {
    pub async fn init(self: Paths) -> Paths {
        if !self.data_dir.exists() {
            std::fs::create_dir_all(&self.data_dir)
                .expect("unable to create user data directory.");
        }
        if !self.config_dir.exists() {
            std::fs::create_dir_all(&self.config_dir)
                .expect("unable to create user config directory.");
        }

        self
    }
}
