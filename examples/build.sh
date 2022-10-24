#!/usr/bin/env bash

# Create res folder if not exists
mkdir -p res

# Iterate over each member
for dir in ./*/*/
do
	echo $dir
        # Run build.sh for each member if exsists
	if [ -f "$dir/build.sh" ]; then
		cd $dir
		./build.sh
		cd ../..
                # Copy wasm file
		cp $dir/target/wasm32-unknown-unknown/release/*.wasm ./res/
	fi
done
