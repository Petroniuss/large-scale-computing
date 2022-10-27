#!/bin/bash

script="#!/bin/bash \n\
#SBATCH -A plglscclass-cpu \n\
#SBATCH -p plgrid \n\
#SBATCH -N 1\n\
#SBATCH --ntasks-per-node=1 \n\
#SBATCH -t 00:01:00 \n\
./large-scale-computing/lab3/lscpuer.sh \n\
exit 0"

curl -k -X POST https://submit.plgrid.pl/api/jobs \
--header "Content-Type:application/json" \
--header "PROXY:$proxy" \
--data '{
    "host":"ares.cyfronet.pl",
    "script":"'"$script"'"
}'

echo ""

