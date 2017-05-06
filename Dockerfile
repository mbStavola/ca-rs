FROM scorpil/rust:nightly

COPY . /rust/ca-rs
WORKDIR /rust/ca-rs

EXPOSE 8080

CMD [ "cargo", "run" ]