/*
 * Copyright [2022] [Kevin Velasco]
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::error::Error;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    if option_env!("WEBPACK_Q_BUILD_JS").is_some() {
        Command::new("npm").current_dir("./js").arg("i").output()?;
        Command::new("npm")
            .current_dir("./js")
            .arg("run")
            .arg("build")
            .output()?;

        std::fs::copy("./js/dist/index.html", "./templates/index.html")?;
    }

    Ok(())
}
