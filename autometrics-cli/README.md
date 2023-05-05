# Autometrics CLI

Currently, the CLI is only used to regenerate the [recording & alerting rules file](https://github.com/autometrics-dev/autometrics-shared#prometheus-recording--alerting-rules).

You will only need to use this if you want to use objective percentiles other than the default set: 90%, 95%, 99%, 99.9%.

To generate the rules file:
1. Clone this repo
    ```sh
    git clone https://github.com/autometrics-dev/autometrics-rs.git
    cd autometrics-rs
    ```
2. Run the CLI to generate a Sloth YAML file:
    ```sh
    cargo run -p autometrics-cli generate-sloth-file -- --objectives=90,95,99,99.9 --output sloth.yml
    ```
3. Run Sloth to generate the Prometheus recording and alerting rules file:
    ```sh
    docker run -v $(pwd):/data  ghcr.io/slok/sloth generate -i /data/sloth.yml -o /data/autometrics.rules.yml
    ```
