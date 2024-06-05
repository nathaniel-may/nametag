# qname

A tag manager that maximizes portability for your media files.

Organizing your media with keywords and tags across file formats usually requires 3rd party system lockin because the app stores your metadata in a separate system. Working with metadata standards such as EXIF isn't possible across all file formats, and many popular tools strip that data because of its inconsitencies. The one thing every file has is a filename. This app lets you define a keword schema, and encodes it in plaintext into the filename so it can be queried by this app or any rudimentary search bar in the system you use to store your files.

## But filenames aren't unlimited!

Local filesystems and cloud services have different restrictions on allowed characters, lengths for filenames, and sometimes lengths for absolute paths. In practice, this isn't severely limiting. APFS and NTFS 3.1 limit the length to 255 unicode characters. That means you could name your file:
```
Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dol.jpg
```
or
```
子曰。學而時習之、不亦說乎。有朋自遠方來、不亦樂乎。人不知而不慍、不亦君子乎。有子曰。其爲人也孝弟、而好犯上者、鮮矣。不好犯上、而好作亂者、未之有也。君子務本、本立而道生。孝弟也者、其爲仁之本與。子曰。巧言令色、鮮矣仁。曾子曰。吾日三省吾身、爲人謀而不忠乎。與朋友交而不信乎。傳不習乎。子曰。道千乘之國、敬事而信、節用而愛人。使民以時。子曰。弟子、入則孝、出則弟、謹而信、凡愛衆、而親仁。行有餘力、則以學文。子夏曰。賢賢易色、事父母、能竭其力、事君、能致其身、與朋友交、言而有信。雖曰未學、吾必謂之學.jpg
```
For other filesystems like ext4 on Linux, the limitation is 255 bytes rather than 255 characters. Using the above examples, these names are 255 bytes when encoded with UTF-8:
```
Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dol.jpg

子曰。學而時習之、不亦說乎。有朋自遠方來、不亦樂乎。人不知而不慍、不亦君子乎。有子曰。其爲人也孝弟、而好犯上者、鮮矣。不好犯上、而好作亂者、未之有也。君子務本、本立而--.jpg
```
And these names are 254 bytes when encoded with UTF-16:
```
Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua..jpg

子曰。學而時習之、不亦說乎。有朋自遠方來、不亦樂乎。人不知而不慍、不亦君子乎。有子曰。其爲人也孝弟、而好犯上者、鮮矣。不好犯上、而好作亂者、未之有也。君子務本、本立而道生。孝弟也者、其爲仁之本與。子曰。巧言令色、鮮矣仁。曾子曰。吾日三省吾身、爲.jpg
```

Table
```
+--------------+-----------------------+--------------------------------------+
|    System    |     Max Filename      | Delimited 7 character tags available |
+--------------+-----------------------+--------------------------------------+
| NTFS 3.1     | 255 characters        | 32                                   |
| APFS         | 255 characters        | 32                                   |
| ext4         | 255 bytes             | <= 32 with UTF-8 encoding            |
| Dropbox      | 255 characters        | 32                                   |
| Google Drive | unlimited characters? | unlimited?                           |
+--------------+-----------------------+--------------------------------------+
```


## Usage

_early stages of development- expect nothing to work and everything to change_

Have a directory with the media files you'd like to organize and include a file named `schema.q` in that directory which describes your desired schema. Run the app from source like so:

```
cargo run -- <path>
```

## Future Features
- Query the filenames that match the schema.
- Rename consecutive sets in the UI.
- Run configuration to skip names that match the schema so you can "pick up where you left off"
- Consolidate configuration

## Build
```
cargo build --release
```

Or for a smaller binary, build with some nightly features. On my laptop it cuts the size down by more than half.
```
cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target aarch64-apple-darwin --release
```

I tried UPX for even smaller artifacts, but it looks like [MacOS 13+ prevents them from running](https://github.com/upx/upx/issues/612)- a seemingly undocumented feature.

## Install
After building from source, I use
```
sudo cp target/aarch64-apple-darwin/release/qname /usr/local/bin/qname
```
