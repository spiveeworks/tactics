#/bin/sh

ip=`hostname -i | sed -e 's/ //'`
port=29563

while true
do
	echo "$ip:$port" | cargo run --release -- -s demo
done

