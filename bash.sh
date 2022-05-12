#!/bin/bash

i=$( ls -l ../web/images/ | grep "^-" | wc -l );

for file in $(ls)
do
        if [ $file != "bash.sh" ]; then
                mv $file $i.png
                i=$(expr $i + 1)
        fi
done

# 该脚本用来批量更改当前文件夹中的名字
# 更改后的名字符合web/images中的命名规律
# 请注意该脚本中的i后的命令路径需要根据实际部署进行更换