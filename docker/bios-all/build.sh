CURRENT_DATE=$(date +"%Y%m%d%H%M")
cd ../../

if [ -z "$TAG" ]; then
    echo "Please enter a tag:"
    read TAG
fi
if [ -z "$TAG" ]; then
    TAG="test-$CURRENT_DATE"
fi

TAG="bios-serv-all:test-$CURRENT_DATE"  # 追加版本号
docker build -f docker/bios-all/Dockerfile -t $TAG .

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
    docker save $TAG -o bios-serv-all.tar
else
    docker push $TAG
fi