# LICENSE-ME

A little CLI-Program for finding ALL possible GIT repositorys on your Machine.
It checks the contents of the folder for an existing "LICENSE" file, and if none is found,
it will present you the directory(ies) where the file is missing.

## Installation

### Build it yourself
Download the Sourcecode, then in the directory execute in terminal:

```bash
cargo run

#or

cargo install
```
### Download pre-built binary's

Download from the release page for your appropriate operating system.

## Usage

### License-me works on BOTH Windows and Unix-like operating Systems!
```bash
# Normal invocation of the Program:

#Windows:
./license-me.exe
#Unix:
./license-me

#additional flags:

#Debug Mode (Verbose + Additional Information)
license-me -d
#Verbose Mode (It prints out nearly everything it does)
license-me -v

#Include repos, where a license already exists, and add another to it!
license-me --append-license
#Include repos, where a license already exists, and replace it!
license-me --replace-license 
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License
[MIT License](https://opensource.org/licenses/MIT)