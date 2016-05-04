
pub static CT_VERT: &'static str = r#"
    #version 140

    in vec3 position;
    in vec3 normal;
    in vec2 texture;

    out vec3 v_normal;
    out vec3 v_position;
    out vec3 frag_position;
    out vec2 v_tex_coords;


    uniform mat4 model;
    uniform mat4 modelview;
    uniform mat4 modelviewperspective;
    uniform mat3 normalmatrix;


    void main() {
        v_normal = normalize(normalmatrix * normal);
        gl_Position = modelviewperspective * vec4(position, 1.0);
        v_position = gl_Position.xyz / gl_Position.w;
        frag_position = vec3(model * vec4(position, 1.0));
        v_tex_coords = texture;
    }
"#;


pub static CT_FRAG: &'static str = r#"
    #version 140
    #define M_PI 3.1415926535897932384626433832795

    in vec3 v_normal;
    in vec3 v_position;

    out vec4 color;
    uniform vec3 ka;
    uniform vec3 kd;
    uniform vec3 ks;

    const vec3 position = vec3(2.0, 4.0, 4.0);

    float Schlick_approx(float VdotH, float spec_reflectance) {
        float exponent = (-5.55473 * VdotH - 6.98316) * VdotH;
        return spec_reflectance + (1 - spec_reflectance) * pow(2, exponent);
    }

    float Schlick(float roughness, float NdotL, float NdotV) {
        float k = pow(roughness + 1, 2) / 8;
        float nl = NdotL / (NdotL * (1 - k) + k);
        float nv = NdotV / (NdotV * (1 - k) + k);

        return nl * nv;
    }

    float GGX_Trowbridge_Reitz(float alpha, float NdotH) {
        float divisor = M_PI * pow(pow(NdotH, 2) * (pow(alpha, 2) - 1) + 1, 2);

        return pow(alpha, 2) / divisor;
    }
    void main() {
         vec3 light_c = vec3(0.4,0.4,0.4);

        vec3 normal = normalize(v_normal);
        vec3 view_dir = normalize(-v_position);
        vec3 light_dir = normalize(position);
        vec3 half_direction = normalize(light_dir +  view_dir);

        //Material parameters used in Physically Based
        //Rendering Model
        float ior = 3;
        float  roughness = 0.41;
        float metallic = 1.0;
        float F0 = abs((1.0 - ior) / (1.0 + ior));
        F0 = F0 * F0;

        F0 = mix(F0, kd.r, metallic);

        //Cook-Torrance Microfacet BRDF as described
        //in Real Shading in Unreal Engine 4
        float alpha = roughness * roughness;
        float NdotL = max(dot(normal, light_dir), 0);
        float spec = 0.0;
        float d_term = 0.0;
        float g_term = 0.0;
        float f_term = 0.0;
        if (NdotL > 0) {
            float NdotV = clamp(dot(normal, view_dir), 0, 1);
            float NdotH = clamp(dot(normal, half_direction), 0, 1);
            float VdotH = clamp(dot(view_dir, half_direction), 0, 1);

            d_term = GGX_Trowbridge_Reitz(alpha, NdotH);
            g_term = Schlick(roughness, NdotL, NdotV);
            f_term = Schlick_approx(VdotH, 1.31);
            spec = clamp(d_term * g_term *  f_term / (4 * NdotL * NdotV), 0, 1);
        }
        color =vec4(clamp(ka +  NdotL * kd / M_PI +  ks * vec3(spec), 0, 1), 1.0);
    }

"#;
pub static CT_FRAG_DIFF: &'static str = r#"
    #version 140
    #define M_PI 3.1415926535897932384626433832795

    in vec3 v_normal;
    in vec3 v_position;
    in vec2 v_tex_coords;


    out vec4 color;
    uniform vec3 ka;
    uniform vec3 kd;
    uniform vec3 ks;
    uniform sampler2D texkd;

    const vec3 position = vec3(0.0, 4.0, 4.0);

    float Schlick_approx(float VdotH, float spec_reflectance) {
        float exponent = (-5.55473 * VdotH - 6.98316) * VdotH;
        return spec_reflectance + (1 - spec_reflectance) * pow(2, exponent);
    }

    float Schlick(float roughness, float NdotL, float NdotV) {
        float k = pow(roughness + 1, 2) / 8;
        float nl = NdotL / (NdotL * (1 - k) + k);
        float nv = NdotV / (NdotV * (1 - k) + k);

        return nl * nv;
    }

    float GGX_Trowbridge_Reitz(float alpha, float NdotH) {
        float divisor = M_PI * pow(pow(NdotH, 2) * (pow(alpha, 2) - 1) + 1, 2);

        return pow(alpha, 2) / divisor;
    }
    void main() {
        vec3 light_c = vec3(0.001,0.007,0.001);
        vec4 tex = texture(texkd, v_tex_coords);

        vec3 normal = normalize(v_normal);
        vec3 view_dir = normalize(-v_position);
        vec3 light_dir = normalize(position);
        vec3 half_direction = normalize(light_dir +  view_dir);

        //Material parameters used in Physically Based
        //Rendering Model
        float ior = 3;
        float  roughness = 0.41;
        float metallic = 1.0;
        float F0 = abs((1.0 - ior) / (1.0 + ior));
        F0 = F0 * F0;

        F0 = mix(F0, kd.r, metallic);

        //Cook-Torrance Microfacet BRDF as described
        //in Real Shading in Unreal Engine 4
        float alpha = roughness * roughness;
        float NdotL = max(dot(normal, light_dir), 0);
        float spec = 0.0;
        float d_term = 0.0;
        float g_term = 0.0;
        float f_term = 0.0;
        if (NdotL > 0) {
            float NdotV = clamp(dot(normal, view_dir), 0, 1);
            float NdotH = clamp(dot(normal, half_direction), 0, 1);
            float VdotH = clamp(dot(view_dir, half_direction), 0, 1);

            d_term = GGX_Trowbridge_Reitz(alpha, NdotH);
            g_term = Schlick(roughness, NdotL, NdotV);
            f_term = Schlick_approx(VdotH, 1.31);
            spec = clamp(d_term * g_term *  f_term / (4 * NdotL * NdotV), 0, 1);
        }
        color = vec4(clamp(NdotL * tex.rgb * kd / M_PI +  ks * vec3(spec), 0, 1), 1.0);
    }

"#;

pub static CT_FRAG_PBR: &'static str = r#"
    #version 140
    #define M_PI 3.1415926535897932384626433832795

    in vec3 v_normal;
    in vec3 v_position;
    in vec3 frag_position;
    in vec2 v_tex_coords;

    out vec4 color;

    struct PointLight {
        vec3 pos;
        vec3 col;
        vec3 attn;
    };

    uniform sampler2D dagger_albedo;
    uniform sampler2D dagger_specular;
    uniform sampler2D dagger_normal;
    uniform sampler2D dagger_gloss;
    uniform float f0;

    uniform Block {
        PointLight lights[5];
    };

    float Schlick_Frensel(float VdotH, float spec_reflectance) {
        return spec_reflectance + (1 - spec_reflectance) * pow(1 - VdotH, 5);
    }
    float Schlick_approx(float VdotH, float spec_reflectance) {
        float exponent = (-5.55473 * VdotH - 6.98316) * VdotH;
        return spec_reflectance + (1 - spec_reflectance) * pow(2, exponent);
    }

    float Schlick_simplified(float x, float NdotL, float NdotV) {
        float k = pow(x + 1, 2) / 8;
        float nl = 1 / (NdotL * (1 - k) + k);
        float nv = 1 / (NdotV * (1 - k) + k);

        return nl * nv / 4;
    }

    float GGX_Trowbridge_Reitz(float alpha, float NdotH) {
        float divisor = M_PI * pow(pow(NdotH, 2) * (pow(alpha, 2) - 1) + 1, 2);

        return pow(alpha, 2) / divisor;
    }

    mat3 cotangent_frame(vec3 normal, vec3 pos, vec2 uv) {
            vec3 dp1 = dFdx(pos);
            vec3 dp2 = dFdy(pos);
            vec2 duv1 = dFdx(uv);
            vec2 duv2 = dFdy(uv);
            vec3 dp2perp = cross(dp2, normal);
            vec3 dp1perp = cross(normal, dp1);
            vec3 T = dp2perp * duv1.x + dp1perp * duv2.x;
            vec3 B = dp2perp * duv1.y + dp1perp * duv2.y;
            float invmax = inversesqrt(max(dot(T, T), dot(B, B)));
            return mat3(T * invmax, B * invmax, normal);
    }

    void main() {
        vec4 tex_albedo = texture(dagger_albedo, v_tex_coords);
        vec4 tex_specular = texture(dagger_specular, v_tex_coords);
        vec4 tex_normal = texture(dagger_normal, v_tex_coords);
        vec4 tex_gloss = texture(dagger_gloss, v_tex_coords);

        mat3 tbn = cotangent_frame(v_normal, v_position, v_tex_coords);
        vec3 normal = tbn * (tex_normal.xyz * 2.0 - 1.0);

        vec3 view_dir = normalize(v_position);
        vec3 temp_color = vec3(0.0);

        for (int i = 0; i < 3; i++) {
            vec3 light_dir = normalize(lights[i].pos);
            vec3 half_direction = normalize(light_dir + view_dir);

            float  roughness = tex_gloss.r;

            //Cook-Torrance Microfacet BRDF as described
            //in Real Shading in Unreal Engine 4

            float alpha = roughness * roughness;
            float NdotL = max(dot(normal, light_dir),0);
            float spec = 0.0;

            float NdotV = max(dot(normal, view_dir), 0.001);
            float NdotH = max(dot(normal, half_direction), 0);
            float VdotH = max(dot(view_dir, half_direction), 0);

            float d_term = GGX_Trowbridge_Reitz(alpha, NdotH);
            float g_term = Schlick_simplified(roughness, NdotL, NdotV);
            float f_term = Schlick_approx(VdotH, f0);
            spec = d_term * g_term * f_term;

            float distance = length(lights[i].pos - frag_position);
            float attenuation = 1.0f  / (lights[i].attn[0] + lights[i].attn[1] * distance + lights[i].attn[2] * distance * distance);
            temp_color += vec3(NdotL * lights[i].col * attenuation * (tex_gloss.g * tex_gloss.b * tex_albedo.rgb / M_PI + tex_specular.rgb * spec));
        }
        color = vec4(temp_color, 1.0);
    }

"#;
