Work-stealer
============

[![Build Status](https://travis-ci.com/montekki/stealing-queue.svg?branch=master)](https://travis-ci.com/montekki/stealing-queue)

Basically it modells the work-stealing scheduler where workers can steal work from each other if they find themselves idle.

```
DEBUG - Thread 0 qs.len 1
DEBUG - Got some work!
DEBUG - Got new job in task 0
TASK 0
TASK 0 END
DEBUG - Thread 0 qs.len 1
DEBUG - Got some work!
DEBUG - Got new job in task 0
TASK 1
TASK 1 END
DEBUG - Thread 0 qs.len 1
DEBUG - Got some work!
DEBUG - Got new job in task 0
TASK 2
TASK 2 END
INFO - Too many tasks, spawning a new worker!
DEBUG - Thread 0 qs.len 2
DEBUG - Got some work!
DEBUG - Got new job in task 0
TASK 3
DEBUG - Thread 1 qs.len 2
DEBUG - Nothing is on the local queue for thread 1
DEBUG - Have managed to steal work from queue 0!
DEBUG - Got some work!
DEBUG - Got new job in task 1

```
