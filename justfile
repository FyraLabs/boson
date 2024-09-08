set dotenv-load

run:
    make pack-dev
    $PWD/build/boson run -- "$GAME_PATH" $GAME_ARGS