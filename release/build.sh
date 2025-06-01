#!/bin/bash
# RTS build script for x86-32 or x86-64
# and optimize (strip + upx)
#
#          x     32/64  all debug
# e:  ./build.sh  64      false
clear

# build file
cd ../
echo -n "[build] "
# x86-64
if [ "$1" == "64" ]; then
	builded=true
	script -q -e -c 'cargo build' ./log > /dev/null 2>&1
	if [ $? -ne 0 ]; then
		builded=false
	fi

	sed -i -e '1d;$d' -e '/^$/d' ./log  # remove begin-end, end space line

	# if build error
	if [ "$builded" == "false" ]; then
		cat   ./log  # output log
		rm -f ./log  # remove log
		exit 1
	fi
	if [[ "$2" == "true" ]]; then
		cat ./log  # output log
	fi
	ls &>> ./log
	rm -f  ./log  # remove log
# x86-32
elif [ "$1" == "32" ]; then
	builded=true
	script -q -e -c 'cargo buil --target=i686-unknown-linux-gnu' ./log > /dev/null 2>&1
	if [ $? -ne 0 ]; then
		builded=false
	fi

	sed -i -e '1d;$d' -e '/^$/d' ./log  # remove begin-end, end space line

	# if build error
	if [ "$builded" == "false" ]; then
	    rustup target add i686-unknown-linux-gnu &> ./log2
	    # if no rustup or i686-unknown-linux-gnu
	    if [ $? -ne 0 ]; then
	    	echo "You need to install 'i686-unknown-linux-gnu' using rustup"
			rm -f ./log2  # remove log2
			rm -f ./log   # remove log
	    	exit 1
		fi
		ls &>> ./log2
		rm -f  ./log2  # remove log2

		cat   ./log  # output log
		rm -f ./log  # remove log
		exit 1
	fi
	#
	if [ "$2" == "true" ]; then
		cat ./log  # output log
	fi
	ls &>> ./log
	rm -f  ./log  # remove log
else
  echo "Specify 32 or 64 architecture for assembly"
  exit 1
fi
echo "Everything is fine [linux-x86-$1]"
#
outputPath="target/debug/rts"
if [ "$1" == "32" ]; then
	outputPath="target/i686-unknown-linux-gnu/debug/rts"
fi
if [ -e "$outputPath" ]; then
	# optimize
	strip $outputPath
	# можно исппользовать 
	# upx --best -q -q -q $outputPath
	# или использовать режим --ultra-brute
	# но они сильно режут скорость работы rts

	# move here
	mv $outputPath release
fi
