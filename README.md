# Mobiumata

# WIP blog post

Mobiumata (a concatenation of mobius strip and cellular automata) is a small interactive art-piece that allows folk to play god to 6,000 LED cells wrapped into a mobius strip.

[VIDEO]

When Scott Logic needed something to act as a talking point for a conference booth, as a big fan of all things flashy, shiny and interactive, I jumped at the opportunity to create something engaging that was roughly themed around AI. This post covers the inspiration for the idea, aspects of the fabrication, highlights of the embedded Rust firmware and some unexpected learnings along the way.

# Flashy things

In previous projects I'd had cause to play with "smart" LEDs (e.g. WS2812b) and knew I wanted to find a way to apply them here. Even if you've not come across the name before, if you've seen any kind of colour changing twinkly lights recently, you've most likely bourne witness to their capabilities. There are two reasons I really like using them -

The first is how easy it is to control the "16M" colours they can produce. Previously, controlling the colour of an RGB LED involved carefully controlling the signal to each of the individual colour LEDs to vary the intensity of each channel. Not only did this require very precise timing and but controlling multiple LEDs quickly used up the available outputs on whatever you used to drive the LEDs.

In comparison "smart" LEDs bundle a controller into the LED package. The controller handles the precise timing required to perform the [Pulse-Width-Modulation](https://en.wikipedia.org/wiki/Pulse-width_modulation). You then control the controller by providing colour data as an RGB byte sequence using just a single pin. Which leads me nicely onto...

The second is simple elegance of the protocol. To explain it, let's quickly see how these LEDs are connected (ignoring power/ground connections) -

```
flowchart LR
 A[Controller] --> B(LED 0)
 B --> C(LED 1)
 C -->|...| D(LED N)
``` 

With the above topology there's a few different addressing strategies that could be used. We could give each LED an address and send packets of colour/address tuples but as we're likely changing lots of colours at once, sending all those addresses is quite wasteful. How about instead we broadcast the colours for all addresses in one message?

This helps but we still have the problem of assigning addresses to each LED. What if we could somehow automatically assign an address based on the LEDs distance (in LEDs) from the controller?

It turns out this is not only easy, but it also greatly simplifies the design of the onboard controller. All it has to do is listen for the first colour, consume it (i.e. don't propagate it) and propagate the remaining colours in the sequence.

I love the simplicity of this design and the ease with which you can trade refresh rate against the number of LEDs (i.e. for a fixed bus frequency, increasing the number of LEDs in any chain, increases the message size, which decreases the rate at which you can trasmit messages).

# Shiny things

In searching around online, I found that you can now buy matrices of these "smart" LEDs on flexible circuit boards. A recent Disney World trip had also taught me that while animations look cool on flat surfaces, running the same animation on a curved surface significantly enhances the effect. All without actually requiring any additional complexity, at least not on the software side -

[VIDEO]

Now all I neeeded was a curved surface to mount some of these LED panels on. Playing around with the panel not only gave me an appreciation of an appropriate bend radius (about 25mm), but also how hot they can get when you drive them hard. So this curved surface was also going to have to be made of metal. However, my metal fabrication skills are somewhat lacking, so I was limited to alluminim (soft enough to use wood-working tools with) and some sort of simple shape, but what?

I'm not sure where the inspiration came from but the most organic looking shape that I could think of, that I also felt I had a hope of fabricating, was a mobius strip -

[IMG]

So we've got a bunch of LEDs in the shape of a mobius strip, how do we make this interactive?

# Interactive things

I think it porbably says more about me than anything else, but Game of Life always comes to mind when I see a low resolution display. Unfortunately, the predominant axis of motion in GoL is diagonal and in this case we're considering a display which is only 8 pixels tall. So whilst it would work, I think it would be an even more confusing and chaotic affair than normal.

When discussing my dilema with a colleague, Simon suggested that I look at [Elementary Cellular Automaton](https://en.wikipedia.org/wiki/Elementary_cellular_automaton). If you're already familiar with GoL, then think of ECA as the 1-dimensional version of the 2-dimensional GoL. For everyone else, here's a quick explainer -

https://en.wikipedia.org/wiki/File:One-d-cellular-automate-rule-30.gif

The novelty of both ECA (and GoL) is that from these simple rules, can lead to the emergence of seemingly complex behaviour. 

This was a great suggestion which maps very neatly onto a mobius strip. Each generation of the ECA could be 8 pixels wide and generations themselves could progress infinitely around the strip. It also covers off the interactivity element, attendees could be invited to choose the rule and configure the wrapping behaviour (where the additional pixel comes from when deciding on the next generation state for a pixel at the top or bottom of the strip).

I had some chunky 2-position industrial switches lying around from a previous project so I could immediately picture one of those for each part of the rule. Throw in another 3-position variant to control the wrapping behaviour (zero-fill, wrap top-to-bottom/bottom-to-top or one-fill) and a push button to slow things down temporarily (to ease explanations to, or assist with discoverability for, attendees). This was pretty much building itself!

# Looming deadlines

With the concept settled, reality hit and I was now left with the small matter of actually building it. Feeling more confident in my software abilities than my metalworking, I decided to start with the metalwork.

For the mobius strip, I could reason that we'd need a piece of alluminium the height of the strip, and half the length of the combined widths of the strips. Plus a bit more to join the two ends together. I also knew that I would be bending at 60 degrees (to form a triangle) but I was less sure where I needed to put the bends and whether they should be interior or exterior bends.

I was about to embark on trying to model this in Fusion 360 to work out the measurements I needed, when it dawned on me that there was a much lower tech way (and quicker) way to achieve the same result: paper. If I scaled down the measurements, I could bend a piece of paper into the shape I wanted, mark the folds and then just scale those measurements back up again.

[IMG the original paper strip is lost to time, but here's an artists impression]

With a pattern to follow, I perhaps made the bending the metal a little more physical than it perhaps needed to be, but I pretty much followed the techniques demonstrated in this tutorial video for bending [right angles](https://youtu.be/VwlXL-OhuMU?si=X3Fj7tvvPVzdH1Wh&t=1068) and [curves](https://youtu.be/VwlXL-OhuMU?si=bdpDZSpgBV3tWF59&t=1398).

[VIDEO bending]

It took a little while but I was very happy with the result. It almost felt a shame to cover it up with LEDs!

[IMAGE naked mobiumata]

To allow the mobiumata to stand on a surface, I needed some kind of base. I considered making one but I've always found it's much easier to customise something than it is to build it from scratch, especially when it's not the main focus of a build. There are definite parallels to software development somewhere in there...

Anyhow, after hunting around the IKEA website I found a [BLANDA MATT (bamboo serving bowl)](https://www.ikea.com/gb/en/p/blanda-matt-serving-bowl-bamboo-60214343/) that I figured upended, was about the right size and shape for a base. I attached a leftover piece of alluminium angle to the mobius strip to act as a leg and drilled a hole for it in the base of the bowl. Subsequently and not shown below, I also ordered a short piece of alluminium tube sized to the hole, to allow the cables to be hidden in the leg.

[IMAGE capture from video showing leg, mobius strip and bowl]

For the controller, I found a somewhat matching [TAVELÅN (bamboo bathroom tray)](https://www.ikea.com/gb/en/p/tavelan-tray-50465756/) that again I figured upended, was about the right size and shape for a control box. This time I used my CNC machine to cut out the holes for the switches -

[VIDEO cutting out holes for switches]


# A massive LED strip

With the hardware somewhat in place, or at least the concept proven, it was time to turn to the software side. Whilst I've previously blogged about [my first steps making a Vim clutch with embedded Rust](https://blog.scottlogic.com/2022/12/08/building-a-rusty-vim-clutch.html), if you're not familiar with the basic concepts and terminology (PAC, HAL, BSP), I'd highly recommend this [video](https://www.youtube.com/watch?v=A9wvA_S6m7Y) which does a much better job of explaining them.

Since writing my last post, the Rust language has stablised the use of `async`. This can greatly simplify application code by removing the need to explicitly maintain state machines and poll routines. For example, here's the guts of the code from that post -

```rust
let mut switch_state = switch_pin.is_low().unwrap();

loop {
    usb_dev.poll(&mut [&mut usb_hid]);

    let previous_switch_state = switch_state;
    switch_state = switch_pin.is_low().unwrap();

    match (previous_switch_state, switch_state) {
        (true, false) => {
            info!("normal mode!");
            led_pin.set_low().unwrap();

            send_key_press(&usb_hid, &mut delay, KEY_ESC);
        }
        (false, true) => {
            info!("insert mode!");
            led_pin.set_high().unwrap();

            send_key_press(&usb_hid, &mut delay, KEY_I);
        }
        _ => {}
    }
}
```

And here is the same thing implemented using [Embassy](https://github.com/embassy-rs/embassy) (an `async` runtime for embedded devices) -

```rust
loop {
    switch_pin.wait_for_high().await;
    info!("normal mode!");
    led_pin.set_low();
    send_key_press(&mut usb_hid, KEY_ESC).await;

    switch_pin.wait_for_low().await;
    info!("insert mode!");
    led_pin.set_high();
    send_key_press(&mut usb_hid, KEY_I).await;
}
```

Notice we're no longer polling the USB device and there's no explicit management of the switch state (and `delay` is also effectively global). In this extremely simplistic example, there's not a huge difference in complexity but you don't have to increase the complexity much for `async` to really shine. 

A prime example of this is my original motivation for using Embassy. I wanted to have the mobiumata controllable remotely, necessitating something like WiFi. In the non-`async` world, [there isn't currently a driver available for the WiFi chip on the Raspberry Pi Pico W](https://github.com/rp-rs/rp-hal/issues/376), whereas Embassy has had a functional driver for the last year or so.

With Embassy chosen, the bulk of the code required to show something on the display came from mashing together the [pio_ws2812](https://github.com/embassy-rs/embassy/blob/b4dc406e199a7e4aafcdd601aaef999c6b7ba590/examples/rp/src/bin/pio_ws2812.rs) example to control the LEDs with the [wifi_ap_tcp_server](https://github.com/embassy-rs/embassy/blob/b4dc406e199a7e4aafcdd601aaef999c6b7ba590/examples/rp/src/bin/wifi_ap_tcp_server.rs) example for running a WiFi Acccess Point and listening for TCP packets on the network. With these in place, from my laptop I could change the colour of the whole display or run a random pattern on it.

As a next step, I wanted to move towards a more functional display which could show something more interesting. So I decided to implement `DrawTarget` from `embedded_graphics`, a library for drawing 2D primitives optimised for embedded devices. The tricky bit here was mapping an X/Y "screen" co-ordinate onto the appropriate LED index (noting that the LEDs are physically connected in a zig-zag pattern) -

```rust
pub fn get_index(x: usize, y: usize) -> usize {
    if y % 2 == 1 {
        x + WIDTH * y
    } else {
        (WIDTH - 1 - x) + WIDTH * y
    }
}
```

Earlier, I covered the positives of `async` in Rust. Unfortunately, at this point I hit upon one of the negatives: the ecosystem is still quite fractured (see also [What Color is Your Function?](https://journal.stuffwithstuff.com/2015/02/01/what-color-is-your-function/)). Instead of being able to write out the data as part of the implementation of `DrawTarget::draw_iter`, I had to add a separate `async` function called `flush` so that I could `.await` the result -

```rust
pub async fn flush(&mut self) {
    self.ws2812.write(self.data[0..NUM_LEDS_PER_PIN].iter().copied()).await;
}
```

[VIDEO text scrolling]

With the display in check, it was time to move on to the automata code itself. I was about to dig out the Wikipedia page and engage in some cathartic algorithm work, when I realised it was a prime opportunity to unleash GitHub Copilot. With a bit of steering, it dutifully kicked out [a perfectly serviceable implementation](https://github.com/chrisprice/mobiumata/blob/c742c8f8a52f3e46e5478fedfec27f415ca61892/mobiumata-automaton/src/lib.rs). The crux of which was -

```rust
pub fn next(&self, state: &[bool], next_state: &mut [bool]) {
    assert_eq!(state.len(), next_state.len());

    let len = state.len();

    for i in 0..len {
        let left = self.wrap.left(state, i);
        let center = state[i];
        let right = self.wrap.right(state, i);

        let index = (left as u8) << 2 | (center as u8) << 1 | right as u8;
        next_state[i] = (self.rule.0 >> index) & 1 == 1;
    }
}
```

For the controls, I just needed to expose the state from above i.e. the rule itself (a [newtype](https://doc.rust-lang.org/book/ch19-04-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction) wrapping an 8-bit unsigned integer) and the wrapping behaviour (an enum with values `Wrap`, `Zero` and `One`). A sprinkling of `embassy-sync::Signal` structs to handle marshalling the value between the network task and the main loop, and we end up with the final main loop of the display -

```rust
loop {
    for y_update in 0..HEIGHT {
        if let Some(new_state) = signal.try_take() {
            state = new_state;
            info!("New state: {:?}", state);
        }

        let automaton = ElementaryCellularAutomaton::new(state.wrap, state.rule);
        automaton.next_row(universe, y_update);

        let pixels = universe.iter().enumerate().flat_map(|(y, row)| {
            row.iter().enumerate().map(move |(x, cell)| {
                Pixel(
                    Point::new(y as i32, x as i32),
                    hsv(if *cell { 170 } else { 15 }, 255, 255),
                )
            })
        });

        display.draw_iter(pixels).unwrap();
        display.flush().await;

        ticker.next(state.step).await;
    }
}
```

The [code for the controller](https://github.com/chrisprice/mobiumata/blob/c742c8f8a52f3e46e5478fedfec27f415ca61892/mobiumata-control/src/main.rs#L81) itself is far simpler than the above. It just reads the state of the various switches and if any of them change, broadcasts the new state over the network.

# Close calls

During the build there were a few unexpected twists and turns. The first one probably doesn't need much in the way of explanation -

[VIDEO Crashing the end mill]

I filled the resulting hole with some wood-filler and relied on folk being suitably distracted when they were playing with it, such that they didn't notice it. I was plesently surprised when this turned out to be the case! Another consequence of this mishap was the odd looking engraved text. As I'd just destroyed my only engraving bit (and didn't have time to wait for another), I decided to run the engraving toolpath with a bull-nosed end mill. I thought it looked awful but I was again plesently surprised when folk didn't notice, assumed it was a concious design decision or were too nice to say anything!

* Doubling the refresh rate

A more fundamental problem cropped up when I was testing the combined LED sections, the refresh rate was just too slow to drive the animation at a speed that felt compelling. 

[VIDEO initial animation]

To maintain the look of the piece, I'd assumed that I could only inject the data signal at the end of the strip where it would align with the base (so that the wires could be hidden inside). However, as I started to assemble the piece I realised that because it was a mobius strip, the join between the third and fourth sections would also align perfectly with the base! And, as I covered earlier, if I split the sections here and injected a second data stream at this point, I could double the refresh rate. I was confident I could get the software side to work, but by this point I'd manhandled the sections a lot trying to get them all into place, before pulling them off to bifurcate them, then reattatching them. That made me a lot less confident things were mechanically/electrically holding up (you'll need sound for this video) -

[VIDEO f***ing miracle]

Nevertheless, I made the requisite changes to the firmware. I modified `Display` such that it used two instances of `Ws2812`, running each on separate state machines (within the same PIO block) and DMA channels, and wired up to the appropriate pin.

```rust
let mut display = Display::new(
    Ws2812::new(&mut pio.common, pio.sm0, p.DMA_CH1, p.PIN_27),
    Ws2812::new(&mut pio.common, pio.sm1, p.DMA_CH2, p.PIN_26),
);
```

I then modified the `flush` implementation to push half the pixels out one, and the other half out the other -

```rust
pub async fn flush(&mut self) {
    self.ws2812_1
        .write(self.data[0..NUM_LEDS_PER_PIN].iter().copied())
        .await;
    self.ws2812_2
        .write(self.data[NUM_LEDS_PER_PIN..NUM_LEDS].iter().copied())
        .await;
}
```

Then I excited flashed the firmware and... whilst I was happy to see all the sections were working as expected... it was exactly the same. There was no noticable difference in the refresh rate. I was dismayed. I double-checked that I had indeed severed the link between the two sections of LEDs. I double-checked that I had the pin mappings right. I double-checked the data bifurcation. I was the on the verge of questioning gravity. Before I finally took a break, stepped back from the problem and immediately realised my mistake - I was outputting the data for the first section, waiting for that to finish and then outputting the data for the second section (and waiting for it to finish). That's functionally equivalent to outputting all of the data to a single section which is where I'd started!

Luckily all the ground work was now done to run it all in parallel, all it took was a minor tweak to the code -

```rust
pub async fn flush(&mut self) {
    join(
        self.ws2812_1
            .write(self.data[0..NUM_LEDS_PER_PIN].iter().copied()),
        self.ws2812_2
            .write(self.data[NUM_LEDS_PER_PIN..NUM_LEDS].iter().copied()),
    )
    .await;
}
```

Success!

* Moving/static propagation front (POSSIBLY)

With everything working, it was time for some stress testing. I set it up in the office for the day and invited folk to play with it. All seemed well when I intermitently checked on it, until I happened to catch a glimpse of it from the other side of the office. Somehow, someone had managed to get the mobiumata, with its colour scheme of blue and yellow-ish, to turn magenta. 

With only a day to go to the conference, I was horrified. I knew there nothing in the code that would allow that to happen. As I walked slowly towards the mobiumata, gaze to the floor, I ran through the possibilies in my head. Either they'd managed to crash the program into this very specific state, or there was something wrong getting the data signal to the LEDs, or, or, or... I was back to questioning gravity. Then something rather unexpected happened, I looked up and the mobiumata was back to blue and yellow-ish!

This made even less sense to me. I tested out the controls and everything seemed fine. It was only when I resolved that it must have been a neutrino, retreated back across the office, glanced back and the magenta was back, that I realised what was happening. I'd resdiscovered dithering!

[VIDEO ]

[Dithering]() is effetively how the separate RGB LEDs combine to make the "16M" colours in the first place. Each of the red, green and blue LEDs are physically distinct in the package. However, because they're relatively close together compared to the distance to our eyes, our eyes see the blended colour. In this case, I happened to produce the same effect blending together the adjacent blue and yellow-ish colours because they were relatively close together compared to how far I was away from them.

Playing with the effect in real life was fun because there's a marginal zone whereby your brain is clearly applying some form of hysterisis, maintaining what it had previously perceived until the evidence overwhelming points to a different perception. In this zone you can convince yourself it is either the distinct colours or the blended colour, depending on your expectation.

The last but possibly most shocking discovery, actually happened after the conference. In my haste to put everything together, I'd hot-glued the bottom of the stand into place rather than use a removable fastener. That meant that I'd had to shove all the wiring into the base, then blindly push the metal base into the hole and hope that I didn't accidently chop through any of the wires. Luckily, after a quick test, everything seemed fine.

It wasn't until after the event, while I was making a few firmware tweaks based on learnings from the day, that I removed the metal base from the wooden base to access the micro-controller and realised just how close I'd come to disaster!

[IMG one strand left]

For anyone left worrying, without a looming deadline, I revisited the hot-glue situation before I put it back together again.

# Conclusion

In the end, everything came together on the day and the mobiumata fulfilled its brief of being a flashy, shiny, interactive, conversation starter. And, it continues to do so today in the reception of Scott Logic's Newcastle office. Please feel free to play with it if you're ever passing through.

Projects like this are always satisfying to see through. Not only for the knowledge you expect to pick up along the way (e.g. embedded Rust, async Rust, etc.), but typically more interesting are the tidbits you didn't expect to (e.g. the many failings of the human visual system, the perils of hot-glue, etc.). I also find it really rewarding when I build something that engages others, so for that I'd like to thank my colleagues who were suitably [nerd-sniped](https://xkcd.com/356/) when I setup mobiumata in the office (without explanation). 

One final anecdote from the build which reminded me of my place in the world was when I showed it to my 4-year old daughter. After a few minutes hacking on the firmware, I tempted her into coming to have a look. "Would you like to see something cool? I've built something covered in rainbows and it's got your name on it!". She placated me by getting up from her tea party and following me into the workshop. When she saw it her face immediately lit up. I was so proud. Until she spoke. "Daddy, you've made the sign from Rocky's truck! Now all you need to do to finish it, is to just make it stay green! For those unaquianted with Paw Patrol, she was referring to the recycling side on the side of his recycling truck...

Thanks for taking the time to read to the end. If you'd like to dig deeper into the code behind mobiumata, you can find it all on [GitHub](https://github.com/chrisprice/mobiumata). 





