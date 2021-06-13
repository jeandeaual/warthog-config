# Thrustmaster HOTAS Warthog™ Configuration Tool

![HOTAS Warthog](assets/warthog_hotas.png)

[![build](https://github.com/jeandeaual/warthog-config/workflows/build/badge.svg)](https://github.com/jeandeaual/warthog-config/actions?query=workflow%3Abuild)

Command-line tool that can be used to control the LEDs of a Thrustmaster HOTAS Warthog throttle.

## Download

<https://github.com/jeandeaual/warthog-config/releases>

## Usage

```text
$ ./warthog-config --help
Command-line tool that can be used to control the LEDs of a Thrustmaster HOTAS Warthog throttle.

USAGE:
    warthog-config [FLAGS] [OPTIONS]

FLAGS:
    -b, --backlight    Turn the backlight on
    -h, --help         Prints help information
    -1, --led-1        Turn the first LED on
    -2, --led-2        Turn the second LED on
    -3, --led-3        Turn the third LED on
    -4, --led-4        Turn the fourth LED on
    -5, --led-5        Turn the fifth LED on
    -r, --read-only    Only show the current state, don't change the LEDs
    -V, --version      Prints version information

OPTIONS:
    -i, --intensity <intensity>    Set the intensity of the backlight (0-5, where 0 in off and 5 is the brightest) [default: 2]
```

### Examples

* Disable all the LEDs

    ```sh
    ./warthog-config -i 0
    ```

* Set the backlight and LEDs 1 and 3 with an intensity of 5

    ```sh
    ./warthog-config -b -1 -3 -i 5
    ```

* Set only LED 2 with the default intensity

    ```sh
    ./warthog-config -2
    ```
