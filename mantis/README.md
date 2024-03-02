## poetry Setup

```sh
poetry install
poetry run blackbox
```
## [pyright Installation](https://github.com/microsoft/pyright) 

### Node Install

If you don't have a recent version of node on your system, install that first from [nodejs.org](https://nodejs.org).

Clear the npm cache: 
`npm cache clean -f`

Install Node’s version manager:
`npm install -g n`

Finally, install the latest stable version
`sudo n stable`

### pyright Package Installation
#### Python Package
A [community-maintained](https://github.com/RobertCraigie/pyright-python) Python package by the name of “pyright” is available on pypi and conda-forge. This package will automatically install node (which Pyright requires) and keep Pyright up to date.

`pip install pyright`

#### NPM Package
Alternatively, you can install the command-line version of Pyright directly from npm, which is part of node. If you don't have a recent version of node on your system, install that first from [nodejs.org](https://nodejs.org). 

To install pyright globally:
`npm install -g pyright`

On MacOS or Linux, sudo may be required to install globally:
`sudo npm install -g pyright`

To update to the latest version:
`sudo npm update -g pyright`