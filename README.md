# Cirpack SBC tracing helper

## Cirpack tracing

To enable tracing on Cirpack SBC run following commands:

``` shell
mgt_cscf -name=<name> -i<instance> -debug=<debuglevel> -loglevel=<loglevel>
```
The `name` can be one of the following processes:
* ibcf
* bgcf

`loglevel` can be from 0-3 and `debuglevel` can be from 0-3.


## Functionality
* Allow to search for specific number or IP
* Automatically prepare and cleanup the tracing
* Provide a time window for the test
* Filter out raw SIP from Cirpack specific debug logs
