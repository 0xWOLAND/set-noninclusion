build:
    cd program && cargo prove build --bin fold-program
    cd program && cargo prove build --bin noninclusion-program

run: 
    cd script && cargo run --release