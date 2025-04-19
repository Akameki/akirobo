# Akiriobo
A modern Tetris versus AI.

https://github.com/user-attachments/assets/06aa1073-65ba-4334-9db2-dac6d4f11d0c

https://github.com/user-attachments/assets/af5038e3-0c6f-4360-a363-987f71cd29ae

## Usage
Currently only supports [Botris Battle](https://botrisbattle.com/).
1. Register on the site and obtain an API token.  
2. Clone the repository.
3. Create an `.env` file as shown in `.env.template`.
4. Run `cargo run --release`. Akirobo will start playing once the game begins.

## What is "Modern Tetris"? What is versus?
_**Modern Tetris**_ games typically include mechanics such as:
* **Hard drop** (instantly drop pieces)
* **Lock delay** (a grace period before a piece locks in place after touching the stack)
* **Rotation system with "kicks"** (nudges a piece if rotation fails, checking nearby positions)
* **Bag-based piece randomizer** (guarantees every type of tetromino will show up regularly)
* **Hold box** and **several preview pieces**

_Puyo Puyo Tetris_, _Tetris Effect_, and many other unofficial titles have these mechanics, while Classic Tetris (e.g., NES/Game Boy) does not.

_**Versus**_ is a live multiplayer mode where players play against each other until someone tops out. A player sends "attacks" to their opponent via garbage lines by clearing lines on their own board. Attack strength depends on factors like number of cleared lines, consecutive clears (combos), or special moves (e.g., T-spin, all-spin, perfect clear). Players are tasked with overwhelming opponents while surviving their attacks.

Depending on the exact ruleset, a modern Tetris versus AI like akirobo may play very differently from a human !!1! 🤖
