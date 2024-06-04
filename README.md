# qname

Organizing your media with keywords and tags across file formats traditionally requires 3rd party system lockin because the app stores metadata in a separate system to organize your files. Working within pre-defined metadata such as EXIF isn't possible across all file formats, and many popular tools strip that data because of its inconsitencies. The one thing every file has is a filename. This app lets you define a keword schema, and it encodes it into the filename so it can be queried by this app or whatever system you choose to store your files in. No lock-in, no incompatibilites.

# But filenames aren't unlimited!

Local filesystems and cloud services have different restrictions on allowed characters, lengths for filenames, and sometimes lengths for absolute paths. In practice, this isn't severely limiting. The most common file systems such as APFS, and NTFS 3.1 limit the length to 255 unicode characters. That means you could name your file:
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


# usage

_early stages of development- expect nothing to work_

Have a directory with the media files you'd like to organize and include a file named `schema.q` in that directory which describes your desired schema. Run the app like so:

```
cargo run <path>
```

Future Features
- Query the filenames that match the schema.
- Problem: you stop half way through and want to move out the named ones. Solution:??? (ideas: put renamed ones in another folder? or skip ones that match the schema? but what about going backwards to fix one)

# build
For a smaller binary, build with some nightly features. On my laptop it cuts the size down by more than half.
```
cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target aarch64-apple-darwin --release
```

I tried UPX for even smaller artifacts, but it looks like [MacOS 13+ prevents them from running](https://github.com/upx/upx/issues/612)- an undocumented feature.

# install
On my machine I use
```
sudo cp target/aarch64-apple-darwin/release/qname /usr/local/bin/qname
```
