export NEEDRESTART_MODE=a
export DEBIAN_FRONTEND=noninteractive
sudo apt update && sudo apt install -y build-essential openjdk-18-jre openjdk-18-jdk maven
cargo install --locked hyperfine

mvn package

hyperfine -N --warmup 3 "java -jar target/parser-test-0.1.0.jar ./AS3257.txt"
