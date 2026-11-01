// La vibracion del control en macOS con lo de Apple: el framework GameController encuentra el
// control (DualSense, DualShock 4, Xbox...) y Core Haptics le manda "hapticos": un evento continuo
// con intensidad (que tan fuerte) y nitidez (grave y redondo, o agudo y seco). Es el camino que
// Apple soporta para estos controles (con SDL por HID, el DualSense acepta la orden y no vibra).
// Solo se compila en macOS (ver build.rs); el juego lo llama desde Rust (src/haptica.rs).

#import <Foundation/Foundation.h>
#import <GameController/GameController.h>
#import <CoreHaptics/CoreHaptics.h>
#include <string.h>

static CHHapticEngine *motor = nil;
static GCController *control_actual = nil;
static char ultimo_error[256] = "";
static char nombre[128] = "";

static void anotar(NSString *s) {
    strlcpy(ultimo_error, s ? s.UTF8String : "", sizeof(ultimo_error));
}

// El primer control conectado que trae hapticos.
static GCController *buscar_control(void) API_AVAILABLE(macos(11.0)) {
    for (GCController *c in [GCController controllers]) {
        if (c.haptics != nil) {
            return c;
        }
    }
    return nil;
}

// Empieza a escuchar controles. Los avisos de "se conecto" llegan por la cola principal: la ventana
// del juego (que procesa sus eventos cada cuadro) los hace correr.
int haptica_iniciar(void) {
    if (@available(macOS 11.3, *)) {
        // Que el control siga vibrando aunque la ventana no este al frente (por ejemplo al grabar).
        GCController.shouldMonitorBackgroundEvents = YES;
    }
    [GCController startWirelessControllerDiscoveryWithCompletionHandler:nil];
    [[NSNotificationCenter defaultCenter] addObserverForName:GCControllerDidDisconnectNotification
                                                      object:nil
                                                       queue:nil
                                                  usingBlock:^(NSNotification *aviso) {
                                                      if (aviso.object == control_actual) {
                                                          motor = nil;
                                                          control_actual = nil;
                                                      }
                                                  }];
    return 1;
}

// Cuantos controles ve el framework (tengan o no hapticos).
int haptica_cuantos(void) {
    return (int)[GCController controllers].count;
}

int haptica_hay_control(void) {
    if (@available(macOS 11.0, *)) {
        return buscar_control() != nil;
    }
    return 0;
}

const char *haptica_nombre(void) {
    GCController *c = [GCController controllers].firstObject;
    strlcpy(nombre, c && c.vendorName ? c.vendorName.UTF8String : "", sizeof(nombre));
    return nombre;
}

const char *haptica_error(void) {
    return ultimo_error;
}

// El motor de hapticos del control (se crea una vez y se reinicia solo si el sistema lo para).
static int preparar(void) API_AVAILABLE(macos(11.0)) {
    if (motor != nil && control_actual != nil) {
        return 1;
    }
    GCController *c = buscar_control();
    if (c == nil) {
        anotar(@"no hay control con hapticos");
        return 0;
    }
    CHHapticEngine *m = [c.haptics createEngineWithLocality:GCHapticsLocalityDefault];
    if (m == nil) {
        anotar(@"el control no dio motor de hapticos");
        return 0;
    }
    m.playsHapticsOnly = YES;
    __weak CHHapticEngine *debil = m;
    m.resetHandler = ^{
        NSError *e = nil;
        [debil startAndReturnError:&e];
    };
    NSError *error = nil;
    if (![m startAndReturnError:&error]) {
        anotar(error.localizedDescription);
        return 0;
    }
    motor = m;
    control_actual = c;
    return 1;
}

// Vibra: `intensidad` y `nitidez` de 0 a 1, por `segundos`. Devuelve 0 si se pudo.
int haptica_vibrar(float intensidad, float nitidez, float segundos) {
    @autoreleasepool {
        if (@available(macOS 11.0, *)) {
            if (!preparar()) {
                return -1;
            }
            CHHapticEventParameter *fuerza = [[CHHapticEventParameter alloc] initWithParameterID:CHHapticEventParameterIDHapticIntensity value:intensidad];
            CHHapticEventParameter *filo = [[CHHapticEventParameter alloc] initWithParameterID:CHHapticEventParameterIDHapticSharpness value:nitidez];
            CHHapticEvent *evento = [[CHHapticEvent alloc] initWithEventType:CHHapticEventTypeHapticContinuous
                                                                  parameters:@[ fuerza, filo ]
                                                                relativeTime:0
                                                                    duration:segundos];
            NSError *error = nil;
            CHHapticPattern *patron = [[CHHapticPattern alloc] initWithEvents:@[ evento ] parameters:@[] error:&error];
            if (patron == nil) {
                anotar(error.localizedDescription);
                return -2;
            }
            id<CHHapticPatternPlayer> jugador = [motor createPlayerWithPattern:patron error:&error];
            if (jugador == nil || ![jugador startAtTime:CHHapticTimeImmediate error:&error]) {
                anotar(error.localizedDescription);
                // Que se vuelva a crear el motor la proxima vez (el control pudo reconectarse).
                motor = nil;
                control_actual = nil;
                return -3;
            }
            return 0;
        }
        anotar(@"hace falta macOS 11 o mas nuevo");
        return -4;
    }
}
