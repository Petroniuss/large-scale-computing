#!/bin/bash

#SBATCH -p plgrid-testing
#SBATCH -N 1
#SBATCH --ntasks-per-node=1
#SBATCH -t 00:10

head -n 10 /etc/passwd | cat | tail -n $((10 - $SLURM_ARRAY_TASK_ID)) | head -n 1

