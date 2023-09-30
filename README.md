# LICENSE-ME

#### Idea came from https://github.com/bukinoshita/license-me - thanks for that! =)

A little CLI-Program for finding ALL possible GIT repositorys on your Machine.
It checks the contents of the folder for an existing "LICENSE" file, and if none is found,
it will present you the directory(ies) where the file is missing.

## What to expect:
This program helps you with:

- Licensing your unlicensed projects
- Creating a dummy README.md if there is none
- Updating current licenses in your projects (and change links in your readme)
- Replace existing licenses (and replace the links in your readme)

## What not to expect:

- Updating the repository on GitHub (this you will have to do yourself)

## How the program works

The program starts with searching on all Drives (even your USB if plugged in!)
for folders with a ".git" folder in it, and assumes that there is a valid Git-Repository.

The folder name where the .git directory is in, is assumed as "Project Title" and will be Inserted
in the dummy README.md - otherwise this function will not take effect anywhere.

If the Program found a README.md in the "repository" and no "LICENSE" file, then
it will create a License for you, appending the License Link to the **end** of the Readme file.

Also, if you wanted to append a second (or third, or fourth....) license to your Project it will create a 
File named like this: "LICENSE-SHORTNAME" and add a link to the **end** of the Readme.md that was found.

(This behaviour may change in the Future - but for now be aware of it!)

If you want to replace a current license the Program will present you all directories with and without a License.
You should know which repository you want to work in. It will read the README.md and searches for
a "## License" section. If the Program cant find one, there will happen nothing, you have to link the License yourself.

Then it will split the README.md file into sections separated by "##" and replaces the "## License" part with a new, formatted
License section. Then the README.md will be overwritten. So there **SHOULD** be no changes made to other parts of your README.

## Installation

### Build it yourself
Download the Sourcecode, then in the directory execute in terminal:

```bash
cargo install --path .
```

### Install Using Cargo

```bash
cargo install --git https://github.com/frequency403/license-me.git
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
#Include all repos assumed as a git repository!
license-me --show-all 
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please feel free to contact me if there is a problem with understanding the Code, or general questions about the Program.

## License
[MIT](http://choosealicense.com/licenses/mit/)