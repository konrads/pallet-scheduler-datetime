# Datetime Scheduler

[![test](https://github.com/konrads/pallet-scheduler-datetime/workflows/test/badge.svg)](https://github.com/konrads/pallet-scheduler-datetime/actions/workflows/test.yml)

A module for unixtime based scheduling.

## Overview

This pallet provides capability to dispatch schedules with a starting UTC time,
optional ending UTC time, and periods consisting of multiples of year/month/week/day/hour/minute/second/ms.
These scheduled dispatches may be named or anonymous and may be canceled.

Scheduling is done via [chrono-light](https://crates.io/crates/chrono-light) library.

**NOTE:** The scheduled calls will be dispatched with the default filter
for the origin: namely `frame_system::Config::BaseCallFilter` for all origin
except root which will get no filter. And not the filter contained in origin
use to call `fn schedule`.

If a call is scheduled using proxy or whatever mechanism which adds filter,
then those filter will not be used when dispatching the schedule call.

## TODO

- backfill runs in case of downtime
- fix weights

## Interface

### Dispatchable Functions

- `sync_scheduleds` - recalculate scheduled wake triggers, accounting for
  potential clock drift.
- `schedule` - schedule a dispatch, which may be periodic, to occur at a
  specified block and with a specified priority.
- `cancel` - cancel a scheduled dispatch, specified by block number and
  index.
- `schedule_named` - augments the `schedule` interface with an additional
  `Vec<u8>` parameter that can be used for identification.
- `cancel_named` - the named complement to the cancel function.

License: Unlicensed

## Substrate runtime wiring

Runtime utilizing this pallet is exemplified in [substrate-node-playground](https://github.com/konrads/substrate-node-playground).
