# Funny stoopid text generation app

We've randomly decided to play with [Markov chains](https://en.wikipedia.org/wiki/Markov_chain) on my discord server, and I decided to release my results in github.

> If you have any questions - refer to `markov-chains --help` or `markov-chains <command> --help`.

## Simple example

1. Build the model from the input text files

> cargo run -- model from-scratch --messages inputs/json/kleden.txt --output outputs/models/kleden1.model

Accepted input files formats are json strings and plain text lines:

Json strings lines:

> "Selamat Pagi"\
> "Minecraft ugh"

Plain text lines:

> Political economy belongs to the category of the social sciences.\
> The basis of the life of society is material production.

2. Load model

> cargo run -- model load --model outputs/models/kleden1.model

There's a bunch of params you can change to play with the model. Most important ones are `--context-window` which configures the "intelligence" of the model, and `--min-length` which can force model to keep generating new text.

## Complex example

1. Generate messages bundle. Those are filtered lists of pre-processed words

> cargo run -- messages parse --path inputs/json/kleden.txt --output outputs/messages/kleden.bundle
> 
> cargo run -- messages parse --path inputs/text/political-economy.txt --output outputs/messages/political-economy.bundle
> 
> cargo run -- messages parse --path inputs/text/state-and-revolution.txt --output outputs/messages/state-and-revolution.bundle

2. Merge books to one messages set

> cargo run -- messages merge --path outputs/messages/political-economy.bundle --path outputs/messages/state-and-revolution.bundle --output outputs/messages/background.bundle

3. Create tokens from the messages sets

> cargo run -- tokens parse --path outputs/messages/kleden.bundle --output outputs/tokens/kleden.bundle
> 
> cargo run -- tokens parse --path outputs/messages/background.bundle --output outputs/tokens/background.bundle

4. Merge tokens to the single bundle

> cargo run -- tokens merge --path outputs/tokens/background.bundle --path outputs/tokens/kleden.bundle --output outputs/tokens/tokens.bundle

5. Tokenize prepared messages bundles

> cargo run -- messages tokenize --messages outputs/messages/background.bundle --tokens outputs/tokens/tokens.bundle --output outputs/tokenized/background.bundle
> 
> cargo run -- messages tokenize --messages outputs/messages/kleden.bundle --tokens outputs/tokens/tokens.bundle --output outputs/tokenized/kleden.bundle

6. Create new dataset from the background messages bundle

> cargo run -- dataset create --messages outputs/tokenized/background.bundle --tokens outputs/tokens/tokens.bundle --output outputs/datasets/kleden2.bundle

7. Extend this dataset with the kleden's messages bundle with bigger weight (10)

> cargo run -- dataset add-messages --path outputs/datasets/kleden2.bundle --messages outputs/tokenized/kleden.bundle --weight 10 --output outputs/datasets/kleden2.bundle

8. Build the model

> cargo run -- model build --dataset outputs/datasets/kleden2.bundle --output outputs/models/kleden2.model

9. Load model

> cargo run -- model load --model outputs/models/kleden2.model

Author: [Nikita Podvirnyi](https://github.com/krypt0nn)\
Licensed under [MIT](LICENSE)
