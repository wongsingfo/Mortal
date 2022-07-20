set -x
set -e
id="$1"
[ -z "$id" ] && echo "$0 <id>" && exit 1
filename=mjlog_pf4-20_n"$id"
wget https://tenhou.net/0/log/"$filename".zip
unzip "$filename".zip
dirname=tenhou_n"$id"
mkdir "$dirname"
ls "$filename" -1 | cut -c -31 | xargs -n 1 -I{} sh -c "[ -e ${dirname}/{}.json.gz ] || ./mjai-reviewer --no-review -t {} --mjai-out - | gzip > ${dirname}/{}.json.gz"
pushd "$dirname"
	ls -1 -l -S | awk '$5<=20 {print $NF}' | xargs rm | :
popd
rm "$filename".zip
rm -r "$filename"

