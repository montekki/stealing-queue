Work-stealer
============

[![Build Status](https://travis-ci.org/montekki/stealing-queue.svg?branch=master)](https://travis-ci.org/montekki/stealing-queue)

Basically it modells the work-stealing scheduler where workers can steal work from each other if they find themselves idle.

```
Thread 0 qs.len 1
Got some work!
Got new job in task 0
TASK 0
TASK 0 END
Thread 0 qs.len 1
Got some work!
Got new job in task 0
TASK 1
TASK 1 END
Thread 0 qs.len 1
Got some work!
Got new job in task 0
TASK 2
TASK 2 END
Too many tasks, spawning a new worker!
Thread 0 qs.len 2
Got some work!
Got new job in task 0
TASK 3
Thread 1 qs.len 2
Nothing is on the local queue for thread 1
Have managed to steal work from queue 0!
Got some work!
Got new job in task 1
TASK 4
TASK 3 END

```
