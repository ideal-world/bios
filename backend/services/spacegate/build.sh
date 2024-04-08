REPO=$TAG
### Rust
if [ -z "$TARGET" ]; then
    echo "Please choose an target:"
    target_options=("debug" "release")
    select opt in "${target_options[@]}"
    do
        case $opt in
            "debug")
                echo "debug"
                TARGET="debug" 
                break
                ;;
            "release")
                echo "release"
                TARGET="release"
                break
                ;;
            *) 
                echo "invalid option"
                ;;
        esac
    done
fi


if [ -z "$TARGET" ] || [ "$TARGET" = "debug" ]; then
    RUST_BUILD_ARG=""
    TARGET_DIR="debug"
else
    RUST_BUILD_ARG="--release"
    TARGET_DIR="release"
fi

# cargo update;
cargo build $RUST_BUILD_ARG;
mv ../../../target/$TARGET_DIR/bios-spacegate ./


### Docker

if [ -z "$TAG" ]; then
    echo "Please enter a tag:"
    read TAG
fi
docker build -t $TAG ./
docker push $TAG