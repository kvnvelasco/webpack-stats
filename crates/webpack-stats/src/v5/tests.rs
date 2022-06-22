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

static SOURCE_FILE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/test_projects/v5/compilation-stats.json"
));

#[test]
fn full_deserialization() {
    let value: super::Stats =
        serde_json::from_str(SOURCE_FILE).expect("Does serde");
}
