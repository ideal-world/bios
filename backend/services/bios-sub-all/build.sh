REPO=$TAG
CURRENT_DATE=$(date +"%Y%m%d%H%M")
### Rust
if [ -z "$TARGET" ]; then
    echo "Please choose a target:"
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
    RUST_BUILD_ARG="--release -p bios-sub-serv-all"
    TARGET_DIR="release"
fi

# cargo update;
cargo build $RUST_BUILD_ARG;
mv ../../../target/$TARGET_DIR/bios-sub-serv-all ./


### Docker

if [ -z "$TAG" ]; then
    echo "Please enter a tag:"
    read TAG
fi
if [ -z "$TAG" ]; then
    TAG="test-$CURRENT_DATE"
fi
docker build -t $TAG ./


if [ -z "$OUTPUT" ]; then
    echo "Where do you want to output:"
    target_options=("tar" "push")
    select opt in "${target_options[@]}"
    do
        case $opt in
            "tar")
                echo "tar"
                OUTPUT="tar" 
                break
                ;;
            "push")
                echo "push"
                OUTPUT="push"
                break
                ;;
            *) 
                echo "invalid option"
                ;;
        esac
    done
fi


if [ -z "$OUTPUT" ] || [ "$OUTPUT" = "tar" ]; then
    docker save $TAG -o bios-sub-serv-all.tar
else
    docker push $TAG
fi

