#!/bin/bash -xe

for filename in shaders/* ; do
  if [ -f $filename ] ; then
    touch "shaders/spirv/${filename#*/}.spv"
    ./spirv.sh "shaders/spirv/${filename#*/}.spv" "$filename"
  fi
done
