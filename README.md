# Funny stoopid text generation app

We've randomly decided to play with [Markov chains](https://en.wikipedia.org/wiki/Markov_chain) on my discord server, and I decided to release my results in github.

## How to use

> If you have any questions - refer to `markov-chains --help` or `markov-chains <command> --help`.

1. Generate messages bundle. Those are filtered lists of pre-processed words

Command: `markov-chains messages parse --path <input file> [--path <input file 2> ...] --output <output file>`
Example: `markov-chains messages parse --path kleden.txt --output kleden.bundle`

Accepted input files formats are json strings and plain text lines:

Json strings lines:

> "Selamat Pagi"\
> "Minecraft ugh"

Plain text lines:

> Political economy belongs to the category of the social sciences.\
> The basis of the life of society is material production.

2. Create tokens from the parsed messages

Command: `markov-chains tokens parse --path <input file> --output <output file>`
Example: `markov-chains tokens parse --path kleden.bundle --output tokens.bundle`

3. Tokenize parsed messages using generated tokens

Command: `markov-chains messages tokenize --messages <input file> --tokens <tokens file> --output <output file>`
Example: `markov-chains messages tokenize --messages kleden.bundle --tokens tokens.bundle --output tokenized-kleden.bundle`

4. Build language model

Command: `markov-chains model build --messages <input file> --output <output file>`
Example: `markov-chains model build --messages tokenized-kleden.bundle --output model.bundle`

5. Load language model

Command: `markov-chains model load --model <model file> --tokens <tokens file> [model params]`
Example: `markov-chains model load --model model.bundle --tokens tokens.bundle`

There's a bunch of params you can change to play with the model. Most important ones are `--context-window` which configures the "intelligence" of the model, and `--min-length` which can force model to keep generating new text.

Author: [Nikita Podvirnyi](https://github.com/krypt0nn)\
Licensed under [MIT](LICENSE)
