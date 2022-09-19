# johnnydecimal

This repository contains several tools for working with [Johnny Decimal](https://johnnydecimal.com) 
systems.

## Installation

### Step 1: Install Johnnydecimal

<details>
<summary>From source</summary>

1. Install rust. Go to the [rust installation page](https://www.rust-lang.org/tools/install)
and follow the instructions.

2. Clone the repository.

``` sh
git clone https://github.com/Tallented-code-bot/johnnydecimal
cd johnnydecimal
```

3. Compile the project.

``` sh
cargo build --release
```

4. Move the binary to somewhere on your `path`. For example, you might move
it to `~/.local/bin/` if you are on Linux.  **Important: the binary is called `jd`, not `johnnydecimal`.**

``` sh
mv ./target/release/jd ~/.local/bin/
```
</details>


### Step 2: Add Johnnydecimal to your shell
This enables you to easily `cd` to any johnnydecimal number.
If you do not want this, you can skip it.

<details>
<summary>Fish</summary>

Add this to your configuration, usually found at `~/.config/fish/config.fish`.

``` fish
jd init fish | source
```
</details>
<details>
<summary>Bash</summary>
Add this to your configuration, usually found at `~/.bashrc`.

``` sh
eval "$(jd init bash)"
```
</details>

<details>
<summary>Zsh</summary>
Add this to your configuration, usually found at `~/.zshrc`.

``` sh
eval "$(jd init zsh)"
```
</details>

## Getting started

The first thing you need to do is index your Johnny Decimal system.

1. Go to the folder above your Johnny Decimal root folder. For example, if your root folder is `~/jd`,
you would go to `~`. 

2. Run `jd index <ROOT_FOLDER>`. For this example, you would run `jd index jd/`. This will look at all
your files and write the index to `<ROOT_FOLDER>/.JdIndex`.

3. As long as you are inside your root folder, you can use `jd`. You could show all your JD numbers with
`jd show`, you could add a new one with `jd add <CATEGORY>`, or you could go to a specific one with
`j <JD_NUMBER>`(assuming that you added the config to your shell!).


## Contributing and License
This project is under the GNU GPL-3 license.  (You can view the license [here](LICENSE).)

Feel free to contribute to this project, either by submitting a pull request
or by adding an issue.

