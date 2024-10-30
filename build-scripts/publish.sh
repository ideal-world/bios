VERSION=$1
PACKAGE_NAME=$2
cargo search "$PACKAGE_NAME" --registry crates-io | grep -q "$PACKAGE_NAME = \"$VERSION\""
if [ $? -eq 0 ]; then
    echo "Version $VERSION of $PACKAGE_NAME is already published."
else
    echo "Publishing version $VERSION of $PACKAGE_NAME"
    cargo package -p $PACKAGE_NAME
    cargo publish -p $PACKAGE_NAME --registry crates-io
fi
