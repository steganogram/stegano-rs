<a name=""></a>
##  (2020-07-10)


#### Bug Fixes

* **FilterReader:**  eof counter has now a state ([d8520e4e](d8520e4e))
* **README:**  little nits ([50f24136](50f24136))
* **RawDecoder:**  use the right decoder to fix ([3f1e0e4d](3f1e0e4d))
* **bench:**  fix crate name ([9d8e72ce](9d8e72ce))
* **cargo:**  licence was not shown as GPL v3 ([8ca30d38](8ca30d38))
* **ci:**
  *  path for the linting script ([83736750](83736750))
  *  false conditions should not break the build ([39d644d4](39d644d4))
  *  add build step to get the missing deps folder generated ([f40fef0a](f40fef0a))
  *  fix permissions on ci scripts ([a08a12a8](a08a12a8))
  *  skip other platfors and rust versions for now ([235da871](235da871))
  *  fix parse errors on travis ci ([152ae68c](152ae68c))
* **ci:coverage:**  adapt pattern to find the test binaries only ([ac9bd94d](ac9bd94d))
* **cli:**  add the cli ([44e9c421](44e9c421))
* **codec:**  legacy content format version 2 ([96078594](96078594))
* **codec:write:**  make sure only written bytes are summed up ([d5a5ce24](d5a5ce24))
* **encoder:**  2 test cases broke during refactoring of multi files ([3e7e5ad2](3e7e5ad2))
* **encoder:files:**  files are not more full paths, only filenames ([cb9d798e](cb9d798e))
* **message:**  make sure that the message only feature is still working ([4e3b888a](4e3b888a))
* **resource:naming:**  fix typo in folder name ([3f4783f2](3f4783f2))
* **test:**
  *  disable the raw message test, fix binary unveil testcase ([30dfba94](30dfba94))
  *  wrong file was used for the testcase ([dc058dc6](dc058dc6))
* **tests:**
  *  fix crate name ([006c15e7](006c15e7))
  *  cleanup experimental bencher code snippets ([5898c580](5898c580))

#### Features

* **BitIterator:**  implemented ([2552039b](2552039b))
* **ContentVersion2:**
  *  support reading content version 2 ([098b03a3](098b03a3))
  *  support saving in content version 2 ([e8904032](e8904032))
* **RawDecoding:**  impl raw message decoding ([8630b98b](8630b98b))
* **cd:**  first draft for a github workflow based artifact deployment to github release ([d903511c](d903511c))
* **cd:artifacts:**  build platform specific binary packages ([05d4b6ff](05d4b6ff))
* **ci:**
  *  add paralell build for osx ([e131072d](e131072d))
  *  publish artifacts on github only when a tag is present ([493b8305](493b8305))
  *  publish artifacts on github ([467f485d](467f485d))
  *  linting as script, no matter what ([d588c3be](d588c3be))
  *  add coverage generation and upload ([9fb12651](9fb12651))
  *  add a build script for linting ([5f8e86c1](5f8e86c1))
  *  add more rust versions and linux as build platforms ([c81d01c9](c81d01c9))
* **ci:coverage:**
  *  testing if the damn coverage on travis is actually working ([b9fa780f](b9fa780f))
  *  switch to tarpaulin for coverage metrics ([93287dc7](93287dc7))
* **ci:post-deploy:**
  *  let's try out tagging ([a547cd8a](a547cd8a))
  *  publish stegano-core on crates.io ([7d814d40](7d814d40))
* **cipher:decoder:**  WIP ([5aa6a512](5aa6a512))
* **cli:**
  *  multiple data files and text message ([8ef76716](8ef76716))
  *  pack unveil and hide in one binary `stegano` ([08db2893](08db2893))
* **cli:ContentVersion2:**  mark the force flag as experimental ([63ca8897](63ca8897))
* **codec:**  draft a codec for version 0x04 that reads a payload size header ([265e6791](265e6791))
* **content:**  impl read of v1 contents ([21fdd3c2](21fdd3c2))
* **content:TextContent:**  implement TextContent and tests for it ([5e5871a7](5e5871a7))
* **coverage:badge:**  add the coverage badge to the README.md ([ad13127c](ad13127c))
* **decoder:**
  *  implement a filter reader that does: ([1364f03c](1364f03c))
  *  v3 decoder that implements the read trait ([2322d462](2322d462))
* **decoder:raw:**  implement raw unveil ([e84bae3f](e84bae3f))
* **decoding:files:**  support multiple files to extract ([c9c90037](c9c90037))
* **encoder:zip:**  zip multiple files before encoding ([d21a4caa](d21a4caa))
* **hide:**  draft of hide cli program ([0d7ba082](0d7ba082))
* **message:**  change compression method to deflate to be compatible with content version 2 ([08dc7fb5](08dc7fb5))
* **raw decoder:**  use new ByteReader for doing the decoding of image data ([9d82512a](9d82512a))
* **test:**  add a dedicated test case for zip file as data file (double zipping test case) ([20473ada](20473ada))
* **unhide:**
  *  implement unhide almost there, only termination must be handled ([4b9db11b](4b9db11b))
  *  implement test for unhide, refactor SteganoDecoder datastructure ([f58f27f0](f58f27f0))
* **unveil:**
  *  decode v2 stegano format ([006b8758](006b8758))
  *  unveil works now to counter part the encoder (not yet for v2.x images) ([55dd1899](55dd1899))
* **workspaces:**
  *  final adjustments ([769c97cd](769c97cd))
  *  travis ci adjustments after refactoring ([83f115d3](83f115d3))
  *  fix relative path issues for ressources ([d61cfb16](d61cfb16))
  *  integrate stegano-cli as a workspace project ([4572a559](4572a559))
  *  move to workspaces crate layout ([98f558a2](98f558a2))



