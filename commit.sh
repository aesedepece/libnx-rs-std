#!/bin/bash
for file in $(git diff | grep "+++ b/src" | sed 's/.*b\/\(.*\)/\1/g'); do
	git add $file && git commit -m "$file"
done
