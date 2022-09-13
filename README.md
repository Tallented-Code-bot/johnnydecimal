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
