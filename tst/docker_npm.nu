export def --wrapped npm [...rest] {
    (
        docker run
        -it
        --rm
        --env UID=(id -u)
        --env GID=(id -g)
        -v (pwd):/pwd
        -w /pwd
        --entrypoint npm
        node:22-alpine --
        ...$rest
    )
}