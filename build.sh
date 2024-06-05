cd wasm;
for contract in * ; do
    echo "Compiling $contract";
    cd $contract && cargo we build && cp target/wasm32-unknown-unknown/release/$contract.wasm ../../bin/$contract.wasm && cd ..;
done
