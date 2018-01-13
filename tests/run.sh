#!/bin/sh
p="$(dirname "$0")"
cd $p

exe="../target/debug/fxconv"
red=`tput setaf 1`
green=`tput setaf 2`
reset=`tput sgr0`
COUNT=0
t()
{
  msg=$1
  timeframe=$2
  output=$3
  inputs=$4
  ok()
  {
    echo "Test $COUNT ${green}[OK]${reset}: fxconv $timeframe out.temp $inputs"
  }
  fail()
  {
    echo "Test $COUNT ${red}[FAIL]${reset} $1: fxconv $timeframe out.temp $inputs"
  }
  rm -rf stdout out.temp
  if eval "$exe $timeframe out.temp $inputs" > stdout$COUNT; then
    if diff $output out.temp; then
      ok
    else
      fail "Miss-match stdout"
      exit 1
    fi
  else
    fail "exit code"
    exit 1
  fi
  rm -rf stdout out.temp
  rm -rf stdout$COUNT
  COUNT=`expr $COUNT + 1`
}

t "Simple test" 1m out00.csv "in00.csv"
t "gaps" 1s out01.csv "in00.csv"
