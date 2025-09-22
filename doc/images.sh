#!/bin/bash
grep '^battery-icon --level' README.md |
  PATH=target/debug /bin/bash
