#!/bin/bash

#SBATCH -p plgrid
#SBATCH -N 1
#SBATCH --ntasks-per-node=4
#SBATCH -t 00:03:00

xvfb-run -a blender --background -noaudio ripple.blend \
	--render-output ./frame_$SLURM_ARRAY_TASK_ID.png --render-frame $SLURM_ARRAY_TASK_ID

