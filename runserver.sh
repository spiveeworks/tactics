#/bin/sh

ip=`hostname -i`
port=29563

while true
do
	echo "$ip:$port" | cargo run --release -- -s
done

